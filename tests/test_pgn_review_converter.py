from contextlib import redirect_stderr
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools import pgn_converter
from tools import pgn_review_converter
from tools.collection_update import CollectionUpdateError


FIXTURES = Path(__file__).parent / "fixtures" / "pgn-review-converter"


def load_fixture(name):
    return (FIXTURES / name).read_text(encoding="utf-8")


def nodes_by_id(game):
    return {node["id"]: node for node in game["analysis"]["nodes"]}


def resolve_uci_path(game, path):
    nodes = nodes_by_id(game)
    node = nodes[game["analysis"]["root"]]
    for uci in path.split("/"):
        matches = [nodes[child] for child in node.get("children", []) if nodes[child]["move"]["uci"] == uci]
        if len(matches) != 1:
            raise AssertionError((path, uci, matches))
        node = matches[0]
    return node


class PgnReviewConverterTests(unittest.TestCase):
    def test_update_failure_is_reported_without_replacing_existing_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "games.json"
            previous = b"previous review collection\n"
            output.write_bytes(previous)
            stderr = io.StringIO()
            with redirect_stderr(stderr), patch.object(
                pgn_review_converter, "atomic_write_bytes",
                side_effect=CollectionUpdateError("cannot atomically replace collection"),
            ):
                rc = pgn_review_converter.main(
                    [str(FIXTURES / "ordinary.pgn"), "-o", str(output)]
                )
            self.assertEqual(rc, 2)
            self.assertIn("cannot atomically replace", stderr.getvalue())
            self.assertEqual(output.read_bytes(), previous)

    def test_rejected_updates_preserve_existing_collection_and_progress(self):
        for name, message in [
            ("invalid-illegal.pgn", "malformed or illegal PGN"),
            ("duplicate-ids.pgn", "duplicate game ID"),
        ]:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as tmp:
                output = Path(tmp) / "games.json"
                progress = Path(tmp) / "progress.json"
                previous = pgn_review_converter.encode_collection(
                    pgn_review_converter.convert_pgn_text(load_fixture("ordinary.pgn"))
                )
                output.write_bytes(previous)
                progress.write_bytes(b"protected progress bytes")
                stderr = io.StringIO()
                with redirect_stderr(stderr):
                    rc = pgn_review_converter.main([str(FIXTURES / name), "-o", str(output)])
                self.assertEqual(rc, 2)
                self.assertIn(message, stderr.getvalue())
                self.assertEqual(output.read_bytes(), previous)
                self.assertEqual(progress.read_bytes(), b"protected progress bytes")
                self.assertEqual(sorted(p.name for p in Path(tmp).iterdir()), ["games.json", "progress.json"])

    def test_annotation_update_replaces_output_with_stable_ids_and_no_temp_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "games-my-games.json"
            encoded = []
            for name in ["id-stability-a.pgn", "id-stability-b.pgn"]:
                with redirect_stderr(io.StringIO()):
                    self.assertEqual(pgn_review_converter.main(
                        [str(FIXTURES / name), "-o", str(output)]
                    ), 0)
                encoded.append(output.read_bytes())
            self.assertNotEqual(*encoded)
            self.assertEqual(*(json.loads(data)["games"][0]["id"] for data in encoded))
            self.assertEqual(list(Path(tmp).iterdir()), [output])

    def test_failed_atomic_rename_removes_temp_and_preserves_existing_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "games.json"
            output.write_bytes(b"previous bytes")
            with redirect_stderr(io.StringIO()), patch(
                "tools.collection_update.os.replace", side_effect=OSError("rename failed")
            ):
                rc = pgn_review_converter.main(
                    [str(FIXTURES / "ordinary.pgn"), "-o", str(output)]
                )
            self.assertEqual(rc, 2)
            self.assertEqual(output.read_bytes(), b"previous bytes")
            self.assertEqual(list(Path(tmp).iterdir()), [output])

    def test_oversized_update_preserves_existing_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "games.json"
            output.write_bytes(b"previous bytes")
            with redirect_stderr(io.StringIO()), patch.object(pgn_review_converter, "MAX_REVIEW_BYTES", 1):
                rc = pgn_review_converter.main(
                    [str(FIXTURES / "ordinary.pgn"), "-o", str(output)]
                )
            self.assertEqual(rc, 2)
            self.assertEqual(output.read_bytes(), b"previous bytes")

    def test_duplicate_generated_ids_fail_clearly(self):
        with self.assertRaisesRegex(pgn_review_converter.ConversionError, "duplicate game ID 'game-"):
            pgn_review_converter.convert_pgn_text(load_fixture("ordinary.pgn") + "\n\n" + load_fixture("ordinary.pgn"))

    def test_ordinary_pgn_without_custom_tags_uses_shared_tree_and_review_shape(self):
        original_builder = pgn_converter._build_tree
        with patch.object(pgn_converter, "_build_tree", wraps=original_builder) as build_tree:
            collection = pgn_review_converter.convert_pgn_text(load_fixture("ordinary.pgn"))

        self.assertTrue(build_tree.called)
        self.assertEqual(collection["version"], 1)
        self.assertEqual(len(collection["games"]), 1)
        game = collection["games"][0]
        self.assertRegex(game["id"], r"^game-[0-9a-f]{20}$")
        self.assertEqual(game["white"], "Ada Lovelace")
        self.assertEqual(game["black"], "Alan Turing")
        self.assertEqual(game["result"], "*")
        self.assertEqual(game["event"], "Club Night")
        self.assertNotIn("solution", game)
        self.assertEqual(len(game["fen"].split()), 6)
        self.assertEqual(game["analysis"]["nodes"][0]["fen"], game["fen"])
        self.assertEqual(resolve_uci_path(game, "e2e4/e7e5/g1f3")["move"]["san"], "Nf3")

    def test_annotated_unicode_game_preserves_ravs_nags_comments_and_explicit_refs(self):
        game = pgn_review_converter.convert_pgn_text(load_fixture("annotated-unicode.pgn"))["games"][0]
        self.assertEqual(game["white"], "José Raúl")
        self.assertEqual(game["black"], "Александр")
        self.assertEqual(game["source"], "fixture:classics")

        d4 = resolve_uci_path(game, "d2d4")
        self.assertEqual(d4["nags"], [1])
        self.assertTrue(any(span["type"] == "move_ref" for span in d4["content"]))
        self.assertIn("plain Nc3", d4["comment"])

        nf6 = resolve_uci_path(game, "d2d4/g8f6")
        self.assertEqual(nf6["role"], "alternative")
        self.assertNotIn("[%role alternative]", nf6["comment"])
        g6 = resolve_uci_path(game, "d2d4/g8f6/c2c4/g7g6")
        self.assertEqual(g6["role"], "sideline")
        self.assertEqual(g6["comment"], "Nested sideline.")

        all_spans = [span for node in game["analysis"]["nodes"] for span in node.get("content", [])]
        plain_spans = [span for span in all_spans if span["type"] == "text"]
        self.assertTrue(any("plain Nc3" in span["text"] for span in plain_spans))

    def test_multiple_games_and_custom_start_fen_are_supported(self):
        collection = pgn_review_converter.convert_pgn_text(load_fixture("multi.pgn"))
        self.assertEqual(len(collection["games"]), 2)
        self.assertEqual(len({game["id"] for game in collection["games"]}), 2)

        custom = pgn_review_converter.convert_pgn_text(load_fixture("custom-fen.pgn"))["games"][0]
        self.assertEqual(custom["fen"].split()[1], "b")
        queen = resolve_uci_path(custom, "h2h1q")
        knight = resolve_uci_path(custom, "h2h1n")
        self.assertEqual(queen["move"]["san"], "h1=Q+")
        self.assertEqual(knight["role"], "sideline")
        self.assertEqual(len(queen["fen"].split()), 6)

    def test_generated_id_is_stable_across_annotation_only_edits(self):
        first = pgn_review_converter.convert_pgn_text(load_fixture("id-stability-a.pgn"))["games"][0]["id"]
        second = pgn_review_converter.convert_pgn_text(load_fixture("id-stability-b.pgn"))["games"][0]["id"]
        self.assertEqual(first, second)

    def test_tag_and_cli_can_override_generated_id(self):
        tagged = pgn_review_converter.convert_pgn_text(load_fixture("override.pgn"))["games"][0]
        self.assertEqual(tagged["id"], "manual-classic")

        options = pgn_review_converter.ReviewConversionOptions(game_id="cli-override")
        overridden = pgn_review_converter.convert_pgn_text(load_fixture("override.pgn"), options)["games"][0]
        self.assertEqual(overridden["id"], "cli-override")

        with self.assertRaises(pgn_review_converter.ConversionError) as raised:
            pgn_review_converter.convert_pgn_text(load_fixture("multi.pgn"), options)
        self.assertIn("single-game PGN", str(raised.exception))

    def test_duplicate_resulting_ids_are_rejected_with_both_game_contexts(self):
        with self.assertRaises(pgn_review_converter.ConversionError) as raised:
            pgn_review_converter.convert_pgn_text(load_fixture("duplicate-ids.pgn"))
        message = str(raised.exception)
        self.assertIn("game #2", message)
        self.assertIn("duplicate game ID 'duplicate-id'", message)
        self.assertIn("game #1", message)

    def test_malformed_game_fails_before_cli_writes_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "games.json"
            stderr = io.StringIO()
            with redirect_stderr(stderr):
                rc = pgn_review_converter.main(
                    [str(FIXTURES / "invalid-illegal.pgn"), "-o", str(output)]
                )
            self.assertEqual(rc, 2)
            self.assertFalse(output.exists())
            self.assertIn("malformed or illegal PGN", stderr.getvalue())

    def test_encoding_is_byte_stable_and_reports_review_size(self):
        options = pgn_review_converter.ReviewConversionOptions(
            title="Classics",
            collection_source="fixture-suite",
            compact=True,
        )
        collection = pgn_review_converter.convert_pgn_text(load_fixture("annotated-unicode.pgn"), options)
        first = pgn_review_converter.encode_collection(collection, compact=True)
        second = pgn_review_converter.encode_collection(
            pgn_review_converter.convert_pgn_text(load_fixture("annotated-unicode.pgn"), options),
            compact=True,
        )
        self.assertEqual(first, second)
        decoded = json.loads(first)
        self.assertEqual(decoded["title"], "Classics")
        self.assertEqual(decoded["source"], "fixture-suite")

        messages = pgn_review_converter.size_messages(len(first))
        self.assertIn(f"encoded size: {len(first)} bytes", messages)
        self.assertIn("8 MiB (8388608 bytes)", messages[1])
        with self.assertRaises(pgn_review_converter.ConversionError):
            pgn_review_converter.validate_encoded_size(pgn_review_converter.MAX_REVIEW_BYTES + 1)

    def test_puzzle_converter_fixture_remains_byte_for_byte_semantically_unchanged(self):
        puzzle_fixtures = FIXTURES.parent / "pgn-converter"
        pgn = (puzzle_fixtures / "valid-book.pgn").read_text(encoding="utf-8")
        expected = json.loads((puzzle_fixtures / "valid-book.json").read_text(encoding="utf-8"))
        self.assertEqual(pgn_converter.convert_pgn_text(pgn), expected)


if __name__ == "__main__":
    unittest.main()

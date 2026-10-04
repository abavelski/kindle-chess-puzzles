from contextlib import redirect_stderr
import io
import json
from pathlib import Path
import tempfile
import unittest

from tools import pgn_converter


FIXTURES = Path(__file__).parent / "fixtures" / "pgn-converter"


def load_fixture(name):
    return (FIXTURES / name).read_text(encoding="utf-8")


def nodes_by_id(puzzle):
    return {node["id"]: node for node in puzzle["analysis"]["nodes"]}


def resolve_uci_path(puzzle, path):
    nodes = nodes_by_id(puzzle)
    node = nodes[puzzle["analysis"]["root"]]
    for uci in path.split("/"):
        matches = [nodes[child] for child in node.get("children", []) if nodes[child]["move"]["uci"] == uci]
        if len(matches) != 1:
            raise AssertionError((path, uci, matches))
        node = matches[0]
    return node


class PgnConverterTests(unittest.TestCase):
    def test_runtime_end_to_end_fixture_matches_offline_pgn_import(self):
        imported = pgn_converter.convert_pgn_text(load_fixture("valid-book.pgn"))
        runtime_fixture = json.loads(load_fixture("valid-book.json"))
        self.assertEqual(runtime_fixture, imported)

    def test_valid_fixture_converts_main_lines_variations_metadata_and_fens(self):
        collection = pgn_converter.convert_pgn_text(load_fixture("valid-book.pgn"))
        self.assertEqual(collection["version"], 1)
        self.assertEqual([puzzle["id"] for puzzle in collection["puzzles"]], [
            "nested-001", "promotion-black-001", "castle-capture-001"
        ])

        nested = collection["puzzles"][0]
        self.assertEqual(nested["difficulty"], "3")
        self.assertEqual(nested["source"], "fixture:nested")
        self.assertEqual(nested["solution"], ["e2e4", "e7e5", "g1f3", "b8c6"])
        self.assertEqual(
            [node["move"]["uci"] for node in nested["analysis"]["nodes"][1:]],
            ["e2e4", "e7e5", "c7c5", "g1f3", "b8c6", "d7d6", "g1f3", "b8c6"],
        )
        self.assertIn("plain Nd5/e2e4.", nested["description"])
        self.assertTrue(any(span["type"] == "move_ref" for span in nested["description_content"]))
        self.assertTrue(any(span.get("text", "").endswith("plain Nd5/e2e4.") for span in nested["description_content"]))

        e4 = resolve_uci_path(nested, "e2e4")
        self.assertEqual(e4["nags"], [1])
        self.assertEqual(e4["role"], "main")
        self.assertIn("1...e5", e4["comment"])
        self.assertIn("e2e4 in plain text", e4["comment"])
        self.assertTrue(any(span["type"] == "move_ref" for span in e4["content"]))

        c5 = resolve_uci_path(nested, "e2e4/c7c5")
        self.assertEqual(c5["role"], "alternative")
        self.assertEqual(c5["comment"], "Sicilian alternative.")
        self.assertNotIn("[%role alternative]", c5["comment"])
        d6 = resolve_uci_path(nested, "e2e4/c7c5/g1f3/d7d6")
        self.assertEqual(d6["role"], "sideline")
        self.assertEqual(d6["comment"], "Nested sideline.")

        promotion = collection["puzzles"][1]
        self.assertEqual(promotion["fen"].split()[1], "b")
        self.assertEqual(promotion["solution"], ["h2h1q", "a1a2"])
        queen = resolve_uci_path(promotion, "h2h1q")
        knight = resolve_uci_path(promotion, "h2h1n")
        self.assertEqual(queen["move"]["san"], "h1=Q+")
        self.assertEqual(knight["role"], "sideline")
        self.assertEqual(len(queen["fen"].split()), 6)

        castle = collection["puzzles"][2]
        capture = resolve_uci_path(castle, "e2e4/e7e5/g1f3/b8c6/f1b5/a7a6/b5c6")
        castle_node = resolve_uci_path(castle, "/".join(castle["solution"]))
        self.assertIn("x", capture["move"]["san"])
        self.assertEqual(castle_node["move"], {"uci": "e1g1", "san": "O-O"})
        for puzzle in collection["puzzles"]:
            for node in puzzle["analysis"]["nodes"]:
                self.assertEqual(len(node["fen"].split()), 6)

    def test_converter_is_byte_stable_for_identical_input_and_options(self):
        options = pgn_converter.ConversionOptions(title="Fixture book", revision=7, compact=True)
        first = pgn_converter.encode_collection(
            pgn_converter.convert_pgn_text(load_fixture("valid-book.pgn"), options), compact=options.compact
        )
        second = pgn_converter.encode_collection(
            pgn_converter.convert_pgn_text(load_fixture("valid-book.pgn"), options), compact=options.compact
        )
        self.assertEqual(first, second)
        decoded = json.loads(first)
        self.assertEqual(decoded["title"], "Fixture book")
        self.assertEqual(decoded["revision"], 7)

    def test_dangling_reference_fails_with_puzzle_and_field_context(self):
        with self.assertRaises(pgn_converter.ConversionError) as raised:
            pgn_converter.convert_pgn_text(load_fixture("dangling-reference.pgn"))
        self.assertIn("puzzle dangling-001 description", str(raised.exception))
        self.assertIn("dangling move_ref", str(raised.exception))

    def test_ambiguous_reference_fails_with_puzzle_and_field_context(self):
        with self.assertRaises(pgn_converter.ConversionError) as raised:
            pgn_converter.convert_pgn_text(load_fixture("ambiguous-reference.pgn"))
        self.assertIn("puzzle ambiguous-001 description", str(raised.exception))
        self.assertIn("ambiguous move_ref", str(raised.exception))

    def test_illegal_pgn_fails_without_partial_output(self):
        with self.assertRaises(pgn_converter.ConversionError) as raised:
            pgn_converter.convert_pgn_text(load_fixture("invalid-illegal.pgn"))
        self.assertIn("puzzle invalid-001", str(raised.exception))
        self.assertIn("malformed or illegal PGN", str(raised.exception))

    def test_puzzle_id_is_required_unless_generated_ids_are_explicitly_enabled(self):
        text = load_fixture("missing-id.pgn")
        with self.assertRaises(pgn_converter.ConversionError) as raised:
            pgn_converter.convert_pgn_text(text)
        self.assertIn("missing required PuzzleId", str(raised.exception))

        options = pgn_converter.ConversionOptions(allow_generated_ids=True)
        first = pgn_converter.convert_pgn_text(text, options)["puzzles"][0]["id"]
        second = pgn_converter.convert_pgn_text(text, options)["puzzles"][0]["id"]
        self.assertRegex(first, r"^generated-[0-9a-f]{16}$")
        self.assertEqual(first, second)

    def test_tag_names_and_defaults_are_options_without_overwriting_present_tags(self):
        text = """[PuzzleId \"metadata-001\"]\n[BookNote \"Tagged note\"]\n[Level \"Hard\"]\n\n1. Nf3 *\n"""
        options = pgn_converter.ConversionOptions(
            description_tag="BookNote",
            difficulty_tag="Level",
            default_description="Default note",
            default_difficulty="Default level",
            default_source="Default source",
        )
        puzzle = pgn_converter.convert_pgn_text(text, options)["puzzles"][0]
        self.assertEqual(puzzle["description"], "Tagged note")
        self.assertEqual(puzzle["difficulty"], "Hard")
        self.assertEqual(puzzle["source"], "Default source")

    def test_malformed_explicit_directive_fails_but_unmarked_move_text_is_plain(self):
        malformed = """[PuzzleId \"bad-ref-001\"]\n[Description \"Broken [%move_ref e2e4\"]\n\n1. e4 *\n"""
        with self.assertRaises(pgn_converter.ConversionError) as raised:
            pgn_converter.convert_pgn_text(malformed)
        self.assertIn("unterminated move_ref directive", str(raised.exception))

        plain = """[PuzzleId \"plain-001\"]\n[Description \"Nd5 h1=Q+ e2e4 stay plain.\"]\n\n1. e4 *\n"""
        puzzle = pgn_converter.convert_pgn_text(plain)["puzzles"][0]
        self.assertEqual(puzzle["description_content"], [{"type": "text", "text": "Nd5 h1=Q+ e2e4 stay plain."}])

    def test_size_report_includes_legacy_and_phase_two_thresholds(self):
        messages = pgn_converter.size_messages(123)
        self.assertEqual(messages[0], "encoded size: 123 bytes")
        self.assertIn("legacy 256 KiB (262144 bytes)", messages[1])
        self.assertIn("phase-two 8 MiB (8388608 bytes)", messages[1])
        self.assertEqual(len(messages), 2)

        messages = pgn_converter.size_messages(pgn_converter.LEGACY_WARNING_BYTES + 1)
        self.assertTrue(any("exceeds 256 KiB" in message for message in messages))

        messages = pgn_converter.size_messages(pgn_converter.PHASE_TWO_MAX_BYTES + 1)
        self.assertTrue(any("exceeds phase-two 8 MiB" in message for message in messages))

    def test_cli_writes_only_after_success_and_reports_size(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "book.json"
            rc = pgn_converter.main([str(FIXTURES / "valid-book.pgn"), "-o", str(output), "--compact"])
            self.assertEqual(rc, 0)
            self.assertEqual(json.loads(output.read_text(encoding="utf-8"))["version"], 1)

            bad_output = Path(tmp) / "bad.json"
            rc = pgn_converter.main([str(FIXTURES / "invalid-illegal.pgn"), "-o", str(bad_output)])
            self.assertEqual(rc, 2)
            self.assertFalse(bad_output.exists())


    def test_cli_regeneration_reports_fen_changes_and_preserves_progress(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            output = directory / "puzzles-book.json"
            progress = directory / "progress.json"

            self.assertEqual(
                pgn_converter.main([str(FIXTURES / "valid-book.pgn"), "-o", str(output)]),
                0,
            )
            progress_bytes = b'{"version":1,"files":{"puzzles-book.json":{"solved_ids":["nested-001"]}}}\n'
            progress.write_bytes(progress_bytes)

            existing = json.loads(output.read_text(encoding="utf-8"))
            existing["puzzles"][0]["fen"] = existing["puzzles"][1]["fen"]
            output.write_text(json.dumps(existing) + "\n", encoding="utf-8")

            stderr = io.StringIO()
            with redirect_stderr(stderr):
                rc = pgn_converter.main(
                    [str(FIXTURES / "valid-book.pgn"), "-o", str(output), "--revision", "2"]
                )

            self.assertEqual(rc, 0)
            self.assertIn("FEN-changed IDs: nested-001", stderr.getvalue())
            self.assertEqual(progress.read_bytes(), progress_bytes)
            regenerated = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(regenerated["revision"], 2)
            self.assertNotIn("progress", regenerated)

    def test_cli_duplicate_existing_ids_fail_before_overwrite(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "puzzles-book.json"
            output.write_text(
                json.dumps(
                    {
                        "version": 1,
                        "puzzles": [
                            {"id": "dup", "fen": "fen-one"},
                            {"id": "dup", "fen": "fen-two"},
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            before = output.read_bytes()
            stderr = io.StringIO()

            with redirect_stderr(stderr):
                rc = pgn_converter.main(
                    [str(FIXTURES / "valid-book.pgn"), "-o", str(output)]
                )

            self.assertEqual(rc, 2)
            self.assertEqual(output.read_bytes(), before)
            self.assertIn("duplicate IDs: dup", stderr.getvalue())
            self.assertIn("duplicate puzzle IDs block replacement", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()

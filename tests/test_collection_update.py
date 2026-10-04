import json
from pathlib import Path
import tempfile
import unittest

from tools import collection_update


def collection(*puzzles):
    return {"version": 1, "puzzles": list(puzzles)}


def puzzle(puzzle_id, fen, **extra):
    return {"id": puzzle_id, "fen": fen, "solution": ["e2e4"], **extra}


class CollectionUpdateTests(unittest.TestCase):
    def test_reorder_and_comment_only_edits_retain_identity(self):
        before = collection(
            puzzle("a", "fen-a", description="old"),
            puzzle("b", "fen-b"),
        )
        after = collection(
            puzzle("b", "fen-b", description="new", analysis={"version": 1}),
            puzzle("a", "fen-a", description="rewritten"),
        )
        report = collection_update.compare_collections(before, after)
        self.assertEqual(report.added_ids, ())
        self.assertEqual(report.removed_ids, ())
        self.assertEqual(report.duplicate_ids, ())
        self.assertEqual(report.fen_changed_ids, ())

    def test_added_removed_and_fen_changed_ids_are_reported_deterministically(self):
        before = collection(puzzle("z", "fen-z"), puzzle("b", "fen-old"), puzzle("a", "fen-a"))
        after = collection(puzzle("c", "fen-c"), puzzle("b", "fen-new"), puzzle("z", "fen-z"))
        report = collection_update.compare_collections(before, after)
        self.assertEqual(report.added_ids, ("c",))
        self.assertEqual(report.removed_ids, ("a",))
        self.assertEqual(report.fen_changed_ids, ("b",))
        self.assertEqual(
            collection_update.comparison_messages(report),
            [
                "collection update comparison:",
                "  added IDs: c",
                "  removed IDs: a",
                "  duplicate IDs: none",
                "  FEN-changed IDs: b",
            ],
        )

    def test_duplicate_ids_are_reported_and_block_replacement(self):
        before = collection(puzzle("a", "fen-a"))
        after = collection(puzzle("dup", "fen-1"), puzzle("dup", "fen-2"))
        report = collection_update.compare_collections(before, after)
        self.assertEqual(report.duplicate_new_ids, ("dup",))
        with self.assertRaises(collection_update.CollectionUpdateError):
            collection_update.ensure_no_duplicate_ids(report)

    def test_atomic_replacement_never_touches_sibling_progress(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            output = directory / "puzzles-book.json"
            progress = directory / "progress.json"
            output.write_text('{"version":1,"puzzles":[]}\n', encoding="utf-8")
            progress_bytes = b'{"version":1,"files":{"puzzles-book.json":{"solved_ids":["a"]}}}\n'
            progress.write_bytes(progress_bytes)
            generated = collection(puzzle("a", "fen-a"))
            encoded = (json.dumps(generated, sort_keys=True) + "\n").encode("utf-8")

            collection_update.atomic_write_bytes(output, encoded)

            self.assertEqual(output.read_bytes(), encoded)
            self.assertEqual(progress.read_bytes(), progress_bytes)
            self.assertNotIn("progress", json.loads(output.read_text(encoding="utf-8")))
            self.assertEqual(list(directory.glob(f".{output.name}.*.tmp")), [])

    def test_invalid_existing_collection_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "puzzles.json"
            path.write_text("not-json", encoding="utf-8")
            with self.assertRaises(collection_update.CollectionUpdateError):
                collection_update.load_collection_file(path)


if __name__ == "__main__":
    unittest.main()

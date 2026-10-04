import copy
import json
from pathlib import Path
import tempfile
import unittest

from tools import clean_book_descriptions


def book_puzzle(side="black"):
    return {
        "id": "135b" if side == "black" else "135w",
        "fen": f"7k/8/5KQ1/8/8/8/8/8 {'b' if side == 'black' else 'w'} - - 0 1",
        "difficulty": 3,
        "description": f"135 - {side} - 3\nKeep the explanation.\nReference: Author",
        "description_content": [
            {"type": "text", "text": f"135 - {side} - 3\n"},
            {"type": "move_ref", "node": "n1", "label": "1...Qg7"},
            {"type": "text", "text": "\nKeep the explanation.\nReference: Author"},
        ],
        "solution": ["g6g7"],
        "topic": "Тема",
        "analysis": {"nodes": [{"id": "n1", "comment": "135 - black - 3"}]},
    }


class CleanBookDescriptionsTests(unittest.TestCase):
    def test_removes_only_metadata_from_both_description_projections(self):
        for side in ("white", "black"):
            with self.subTest(side=side):
                before = {"version": 1, "puzzles": [book_puzzle(side)]}
                original = copy.deepcopy(before)
                after, changed = clean_book_descriptions.clean_collection(before)
                expected = copy.deepcopy(before)
                expected["puzzles"][0]["description"] = "Keep the explanation.\nReference: Author"
                expected["puzzles"][0]["description_content"].pop(0)
                self.assertEqual(after, expected)
                self.assertEqual(changed, 1)
                self.assertEqual(before, original)
                again, changed = clean_book_descriptions.clean_collection(after)
                self.assertEqual(again, after)
                self.assertEqual(changed, 0)

    def test_preserves_prose_and_nonmatching_metadata(self):
        for changes in (
            {"id": "other"}, {"id": "136b"}, {"difficulty": 4},
            {"fen": "7k/8/5KQ1/8/8/8/8/8 w - - 0 1"},
            {"description": "An authored introduction.\n135 - black - 3", "description_content": []},
            {"description": None, "description_content": []},
        ):
            with self.subTest(changes=changes):
                puzzle = {**book_puzzle(), **changes}
                before = {"version": 1, "puzzles": [puzzle]}
                after, changed = clean_book_descriptions.clean_collection(before)
                self.assertEqual(after, before)
                self.assertEqual(changed, 0)

    def test_legacy_and_combined_first_text_span(self):
        puzzle = book_puzzle()
        del puzzle["description_content"]
        after, changed = clean_book_descriptions.clean_collection({"puzzles": [puzzle]})
        self.assertEqual(changed, 1)
        self.assertNotIn("description_content", after["puzzles"][0])
        puzzle = book_puzzle()
        puzzle["description_content"][0]["text"] += "Authored prose stays."
        after, _ = clean_book_descriptions.clean_collection({"puzzles": [puzzle]})
        self.assertEqual(after["puzzles"][0]["description_content"][0],
                         {"type": "text", "text": "Authored prose stays."})

    def test_metadata_only_fields_do_not_leave_empty_text_spans(self):
        puzzle = book_puzzle()
        puzzle["description"] = "135 - black - 3"
        puzzle["description_content"] = [{"type": "text", "text": "135 - black - 3\n"}]
        after, _ = clean_book_descriptions.clean_collection({"puzzles": [puzzle]})
        self.assertEqual(after["puzzles"][0]["description"], "")
        self.assertEqual(after["puzzles"][0]["description_content"], [])

    def test_file_cleanup_preserves_progress_and_is_byte_stable_on_repeat(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            path = directory / "puzzles-book.json"
            path.write_text(json.dumps({"version": 1, "puzzles": [book_puzzle()]}), encoding="utf-8")
            progress = directory / "progress.json"
            progress.write_bytes(b'untouched progress, including malformed records')
            self.assertEqual(clean_book_descriptions.clean_file(path), 1)
            cleaned_bytes = path.read_bytes()
            self.assertEqual(clean_book_descriptions.clean_file(path), 0)
            self.assertEqual(path.read_bytes(), cleaned_bytes)
            self.assertEqual(progress.read_bytes(), b'untouched progress, including malformed records')
            self.assertEqual(list(directory.glob(".*.tmp")), [])


if __name__ == "__main__":
    unittest.main()

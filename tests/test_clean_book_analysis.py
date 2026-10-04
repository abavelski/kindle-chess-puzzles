import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools import clean_book_analysis


def puzzle(reference="Author Player, 1976"):
    movetext = "1. Qg7# { Author - Player, 1976 } *"
    return {
        "id": "1w", "fen": "7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "solution": ["g6g7"], "difficulty": 1, "topic": "Mate",
        "source": "book.json", "reference": reference,
        "pgn": '[Event "Example"]\n\n' + movetext,
        "description": movetext + ("\nReference: " + reference if reference else ""),
        "description_content": [
            {"type": "move_ref", "node": "n1", "label": "1.Qg7#"},
            {"type": "text", "text": " "},
        ] + ([{"type": "text", "text": "\nReference: " + reference}] if reference else []),
        "analysis": {"version": 1, "root": "n0", "nodes": [
            {"id": "n0", "fen": "7k/8/5KQ1/8/8/8/8/8 w - - 0 1", "children": ["n1"]},
            {"id": "n1", "parent": "n0", "role": "main",
             "move": {"uci": "g6g7", "san": "Qg7#"},
             "fen": "7k/6Q1/5K2/8/8/8/8/8 b - - 1 1",
             "comment": "Author - Player, 1976"},
        ]},
    }


class CleanBookAnalysisTests(unittest.TestCase):
    def test_removes_both_redundant_projections_preserving_every_other_field(self):
        for reference in (None, "", "Author Player, 1976"):
            with self.subTest(reference=reference):
                before = {"version": 1, "title": "Book", "puzzles": [puzzle(reference)]}
                original = copy.deepcopy(before)
                after, count = clean_book_analysis.clean_collection(before)
                expected = copy.deepcopy(before)
                expected["puzzles"][0].update(description="", description_content=[])
                self.assertEqual(after, expected)
                self.assertEqual(count, 1)
                self.assertEqual(before, original)
                self.assertEqual(clean_book_analysis.clean_collection(after), (after, 0))

    def test_keeps_unique_reference_and_respects_authoritative_comment_content(self):
        for change in ({"reference": "Unique source"}, {"reference": "!!"}, {"content": []}):
            with self.subTest(change=change):
                item = puzzle(change.get("reference", "Author Player, 1976"))
                if "content" in change:
                    item["analysis"]["nodes"][1]["content"] = change["content"]
                after, count = clean_book_analysis.clean_collection({"puzzles": [item]})
                text = "Reference: " + item["reference"]
                self.assertEqual(count, 1)
                self.assertEqual(after["puzzles"][0]["description"], text)
                self.assertEqual(after["puzzles"][0]["description_content"], [{"type": "text", "text": text}])

    def test_attribution_can_span_several_rendered_comments(self):
        item = puzzle("Author Player, 1976")
        item["analysis"]["nodes"][0]["content"] = [{"type": "text", "text": "Author - Player"}]
        item["analysis"]["nodes"][1]["content"] = [{"type": "text", "text": "1976"}]
        after, _ = clean_book_analysis.clean_collection({"puzzles": [item]})
        self.assertEqual(after["puzzles"][0]["description_content"], [])

    def test_punctuation_only_attribution_requires_a_literal_match(self):
        item = puzzle("!?")
        item["analysis"]["nodes"][1]["comment"] = "!?"
        after, _ = clean_book_analysis.clean_collection({"puzzles": [item]})
        self.assertEqual(after["puzzles"][0]["description_content"], [])

    def test_preserves_authored_prose_partial_and_non_main_references(self):
        variants = []
        item = puzzle(); item["description"] = "Unique authored explanation"; variants.append(item)
        item = puzzle(); item["description_content"].insert(0, {"type": "text", "text": "Consider "}); variants.append(item)
        item = puzzle(); item["description_content"] = [{"type": "text", "text": "Plain Qg7#"}]; variants.append(item)
        item = puzzle(); item["analysis"]["nodes"][1]["role"] = "sideline"; variants.append(item)
        item = puzzle(); item["analysis"]["nodes"][1]["children"] = ["n2"]
        item["analysis"]["nodes"].append({"id": "n2", "role": "main", "parent": "n1"}); variants.append(item)
        for item in variants:
            with self.subTest(item=item):
                before = {"puzzles": [item]}
                self.assertEqual(clean_book_analysis.clean_collection(before), (before, 0))

    def test_fallback_without_pgn_keeps_original_note_but_removes_summary(self):
        item = puzzle("")
        del item["pgn"]
        item["description"] = "Original source line and variations\nERROR!!!"
        after, count = clean_book_analysis.clean_collection({"puzzles": [item]})
        self.assertEqual(count, 1)
        self.assertEqual(after["puzzles"][0]["description"], item["description"])
        self.assertEqual(after["puzzles"][0]["description_content"], [])

    def test_atomic_file_cleanup_is_idempotent_and_preserves_progress(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            path = directory / "puzzles-book.json"
            path.write_text(json.dumps({"version": 1, "puzzles": [puzzle()]}))
            progress = directory / "progress.json"
            progress.write_bytes(b"malformed future progress preserved")
            with patch("tools.collection_update.os.replace", side_effect=OSError("blocked")):
                original = path.read_bytes()
                with self.assertRaises(ValueError):
                    clean_book_analysis.clean_file(path)
                self.assertEqual(path.read_bytes(), original)
            self.assertEqual(clean_book_analysis.clean_file(path), 1)
            original = path.read_bytes()
            self.assertEqual(clean_book_analysis.clean_file(path), 0)
            self.assertEqual(path.read_bytes(), original)
            self.assertEqual(progress.read_bytes(), b"malformed future progress preserved")
            self.assertEqual(list(directory.glob(".*.tmp")), [])


if __name__ == "__main__":
    unittest.main()

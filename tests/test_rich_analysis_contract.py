import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures" / "rich-analysis"
LEGACY_FIXTURE = ROOT / "tests" / "fixtures" / "puzzles.json"
FORMAT_DOC = ROOT / "docs" / "PUZZLE_FORMAT.md"

UCI_RE = re.compile(r"^[a-h][1-8][a-h][1-8][qrbn]?$")
PHASE_TWO_MAX_BYTES = 8 * 1024 * 1024
LEGACY_WARNING_BYTES = 256 * 1024

INVALID_FIXTURES = {
    "invalid-duplicate-id.json": "duplicate-id",
    "invalid-missing-child.json": "missing-child",
    "invalid-cycle.json": "cycle",
    "invalid-disconnected.json": "disconnected",
    "invalid-root-fen-mismatch.json": "root-fen",
    "invalid-uci.json": "uci",
    "invalid-main-solution-mismatch.json": "main-solution",
    "invalid-move-ref-dangling.json": "move-ref",
    "invalid-move-ref-root.json": "move-ref",
    "invalid-span-missing-text.json": "span",
    "invalid-span-unknown-type.json": "span",
    "invalid-span-bad-label.json": "span",
}

class ContractError(ValueError):
    def __init__(self, code):
        super().__init__(code)
        self.code = code

def fail(code):
    raise ContractError(code)

def normalized_role(node):
    return node.get("role", "sideline")

def validate_spans(spans, nodes, root_id):
    if not isinstance(spans, list):
        fail("span")
    for span in spans:
        if not isinstance(span, dict) or not isinstance(span.get("type"), str):
            fail("span")
        if span["type"] == "text":
            if set(span) != {"type", "text"}:
                fail("span")
            if not isinstance(span["text"], str) or not span["text"]:
                fail("span")
        elif span["type"] == "move_ref":
            if not set(span).issubset({"type", "node", "label"}):
                fail("span")
            if "node" not in span or not isinstance(span["node"], str) or not span["node"]:
                fail("span")
            if "label" in span and (not isinstance(span["label"], str) or not span["label"]):
                fail("span")
            if span["node"] == root_id or span["node"] not in nodes:
                fail("move-ref")
        else:
            fail("span")

def validate_collection(collection):
    if collection.get("version") != 1 or not isinstance(collection.get("puzzles"), list):
        fail("collection")
    for puzzle in collection["puzzles"]:
        analysis = puzzle.get("analysis")
        if analysis is None:
            continue
        if analysis.get("version") != 1 or not isinstance(analysis.get("nodes"), list):
            fail("analysis")
        node_list = analysis["nodes"]
        ids = [node.get("id") for node in node_list if isinstance(node, dict)]
        if len(ids) != len(node_list) or any(not isinstance(node_id, str) or not node_id for node_id in ids):
            fail("node-id")
        if len(set(ids)) != len(ids):
            fail("duplicate-id")
        nodes = {node["id"]: node for node in node_list}
        root_id = analysis.get("root")
        if root_id not in nodes:
            fail("root")
        root = nodes[root_id]
        if root.get("fen") != puzzle.get("fen"):
            fail("root-fen")
        if any(field in root for field in ("parent", "move", "role", "nags")):
            fail("root")
        for node_id, node in nodes.items():
            children = node.get("children", [])
            if not isinstance(children, list) or any(not isinstance(child, str) for child in children):
                fail("children")
            if len(set(children)) != len(children):
                fail("children")
            for child in children:
                if child not in nodes:
                    fail("missing-child")
            if node_id == root_id:
                continue
            if not isinstance(node.get("parent"), str) or node["parent"] not in nodes:
                fail("parent")
            move = node.get("move")
            if not isinstance(move, dict) or set(move) != {"uci", "san"}:
                fail("move")
            if (not isinstance(move["uci"], str) or not UCI_RE.fullmatch(move["uci"]) or move["uci"][:2] == move["uci"][2:4]):
                fail("uci")
            if not isinstance(move["san"], str) or not move["san"]:
                fail("move")
            if not isinstance(node.get("fen"), str) or len(node["fen"].split()) != 6:
                fail("fen")
            if normalized_role(node) not in {"main", "alternative", "sideline"}:
                fail("role")
            if "comment" in node and not isinstance(node["comment"], str):
                fail("comment")
            if "nags" in node:
                nags = node["nags"]
                if not isinstance(nags, list) or any(type(nag) is not int or nag < 0 for nag in nags):
                    fail("nags")

        visiting, visited = set(), set()
        def visit(node_id):
            if node_id in visiting:
                fail("cycle")
            if node_id in visited:
                return
            visiting.add(node_id)
            for child in nodes[node_id].get("children", []):
                visit(child)
            visiting.remove(node_id)
            visited.add(node_id)
        for node_id in nodes:
            visit(node_id)

        reachable = set()
        def mark(node_id):
            if node_id in reachable:
                return
            reachable.add(node_id)
            for child in nodes[node_id].get("children", []):
                mark(child)
        mark(root_id)
        if reachable != set(nodes):
            fail("disconnected")

        child_owner = {}
        for parent_id, parent in nodes.items():
            for child_id in parent.get("children", []):
                if child_id in child_owner:
                    fail("parent")
                child_owner[child_id] = parent_id
        for node_id, node in nodes.items():
            if node_id != root_id and child_owner.get(node_id) != node["parent"]:
                fail("parent")

        projected, main_ids = [], set()
        cursor = root
        while True:
            main_children = [nodes[child_id] for child_id in cursor.get("children", []) if normalized_role(nodes[child_id]) == "main"]
            if len(main_children) > 1:
                fail("main-solution")
            if not main_children:
                break
            cursor = main_children[0]
            main_ids.add(cursor["id"])
            projected.append(cursor["move"]["uci"])
        if projected != puzzle.get("solution"):
            fail("main-solution")
        if any(normalized_role(node) == "main" and node_id not in main_ids for node_id, node in nodes.items() if node_id != root_id):
            fail("main-solution")

        if "description_content" in puzzle:
            validate_spans(puzzle["description_content"], nodes, root_id)
        for node in nodes.values():
            if "content" in node:
                validate_spans(node["content"], nodes, root_id)

def representative_book():
    collection = {"version":1,"revision":1,"title":"Representative rich book","puzzles":[]}
    root_fen = "8/8/8/8/8/8/7p/K5k1 b - - 0 1"
    node_fen = "8/8/8/8/8/8/8/K5q1 w - - 0 2"
    seed = "A positional explanation with concrete comparison and practical notes. "
    comment = (seed * ((96 // len(seed)) + 1))[:96]
    for puzzle_index in range(300):
        nodes = [{"id":"n0","fen":root_fen,"comment":"Start position.","children":["n1","n21","n26"],
                  "content":[{"type":"text","text":"Compare "},{"type":"move_ref","node":"n1"},
                             {"type":"text","text":" with "},{"type":"move_ref","node":"n21","label":"the sideline"}]}]
        for node_index in range(1, 21):
            nodes.append({"id":f"n{node_index}","parent":"n0" if node_index == 1 else f"n{node_index - 1}",
                          "move":{"uci":"h2h1q" if node_index == 1 else "a1a2","san":"h1=Q+" if node_index == 1 else "Ka2"},
                          "fen":node_fen,"role":"main","comment":comment,"nags":[1] if node_index % 7 == 0 else [],
                          "children":[f"n{node_index + 1}"] if node_index < 20 else []})
        for node_index in range(21, 36):
            nodes.append({"id":f"n{node_index}","parent":"n0" if node_index in (21, 26) else f"n{node_index - 1}",
                          "move":{"uci":"h2h1r" if node_index < 26 else "h2h1n","san":"h1=R" if node_index < 26 else "h1=N"},
                          "fen":node_fen,"role":"alternative" if node_index == 26 else "sideline","comment":comment,
                          "children":[] if node_index in (25, 35) else [f"n{node_index + 1}"]})
        collection["puzzles"].append({
            "id":f"book-{puzzle_index + 1:04d}","fen":root_fen,
            "description":"Find the best continuation. Nd5 in plain text is not a link.",
            "description_content":[{"type":"text","text":"Find "},{"type":"move_ref","node":"n1","label":"the main move"},
                                   {"type":"text","text":" and compare the alternatives."}],
            "difficulty":"Book","solution":["h2h1q"] + ["a1a2"] * 19,
            "analysis":{"version":1,"root":"n0","nodes":nodes}})
    return collection

class RichAnalysisContractTests(unittest.TestCase):
    def test_valid_fixture_covers_required_contract_features(self):
        fixture = json.loads((FIXTURES / "valid-rich.json").read_text(encoding="utf-8"))
        validate_collection(fixture)
        puzzle = fixture["puzzles"][0]
        nodes = puzzle["analysis"]["nodes"]
        self.assertEqual(puzzle["fen"].split()[1], "b")
        self.assertTrue(any(node.get("role") == "main" for node in nodes))
        self.assertTrue(any(node.get("role") == "alternative" for node in nodes))
        self.assertTrue(any(node.get("nags") for node in nodes))
        self.assertTrue(any(len(node.get("move", {}).get("uci", "")) == 5 for node in nodes))
        self.assertTrue(any(node.get("comment") for node in nodes))
        self.assertTrue(any(node.get("content") for node in nodes))
        self.assertTrue(any(span["type"] == "move_ref" for span in puzzle["description_content"]))
        defaulted = next(node for node in nodes if node["id"] == "n3")
        for field in ("role", "comment", "content", "nags", "children"):
            self.assertNotIn(field, defaulted)

    def test_invalid_fixtures_fail_for_the_named_contract_reason(self):
        for filename, expected_code in INVALID_FIXTURES.items():
            with self.subTest(filename=filename):
                fixture = json.loads((FIXTURES / filename).read_text(encoding="utf-8"))
                with self.assertRaises(ContractError) as raised:
                    validate_collection(fixture)
                self.assertEqual(raised.exception.code, expected_code)

    def test_legacy_v1_fixture_remains_the_analysis_free_control(self):
        fixture = json.loads(LEGACY_FIXTURE.read_text(encoding="utf-8"))
        self.assertEqual(fixture["version"], 1)
        self.assertTrue(fixture["puzzles"])
        for puzzle in fixture["puzzles"]:
            self.assertNotIn("analysis", puzzle)
            self.assertNotIn("description_content", puzzle)

    def test_reference_directive_and_byte_limits_are_frozen_in_the_format_doc(self):
        doc = FORMAT_DOC.read_text(encoding="utf-8")
        self.assertIn("[%move_ref h2h1q|1...h1=Q+]", doc)
        self.assertIn("8 MiB (8,388,608 bytes)", doc)
        self.assertIn("256 KiB", doc)
        self.assertIn("tests/fixtures/puzzles.json", doc)

    def test_representative_book_size_profile_matches_recorded_measurement(self):
        sample = representative_book()
        compact = json.dumps(sample, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        pretty = json.dumps(sample, ensure_ascii=False, indent=2).encode("utf-8")
        self.assertEqual(len(compact), 2_845_573)
        self.assertEqual(len(pretty), 5_214_393)
        self.assertGreater(len(compact), LEGACY_WARNING_BYTES)
        self.assertLess(len(pretty), PHASE_TWO_MAX_BYTES)

if __name__ == "__main__":
    unittest.main()

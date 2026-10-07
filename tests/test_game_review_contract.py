import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures" / "game-review"
LEGACY_FIXTURE = ROOT / "tests" / "fixtures" / "puzzles.json"
RICH_FIXTURE = ROOT / "tests" / "fixtures" / "rich-analysis" / "valid-rich.json"
FORMAT_DOC = ROOT / "docs" / "GAME_REVIEW_FORMAT.md"

UCI_RE = re.compile(r"^[a-h][1-8][a-h][1-8][qrbn]?$")
RESULTS = {"1-0", "0-1", "1/2-1/2", "*"}
MAX_REVIEW_BYTES = 8 * 1024 * 1024
STANDARD_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

INVALID_FIXTURES = {
    "invalid-duplicate-game-id.json": "duplicate-game-id",
    "invalid-missing-fen.json": "fen",
    "invalid-root-fen-mismatch.json": "root-fen",
    "invalid-duplicate-node-id.json": "duplicate-node-id",
    "invalid-missing-child.json": "missing-child",
    "invalid-cycle.json": "cycle",
    "invalid-disconnected.json": "disconnected",
    "invalid-uci.json": "uci",
    "invalid-node-fen.json": "fen",
    "invalid-empty-main-line.json": "empty-main-line",
    "invalid-move-ref-dangling.json": "move-ref",
    "invalid-solution-field.json": "solution",
}


class ContractError(ValueError):
    def __init__(self, code):
        super().__init__(code)
        self.code = code


def fail(code):
    raise ContractError(code)


def is_review_collection_filename(name):
    if name == "games.json":
        return True
    if not name.startswith("games-") or not name.endswith(".json"):
        return False
    stem = name[len("games-") : -len(".json")]
    return bool(stem) and "/" not in stem and "\\" not in stem


def validate_fen(fen):
    if not isinstance(fen, str):
        fail("fen")
    fields = fen.split()
    if len(fields) != 6:
        fail("fen")

    ranks = fields[0].split("/")
    if len(ranks) != 8:
        fail("fen")
    pieces = set("PNBRQKpnbrqk")
    for rank in ranks:
        files = 0
        for symbol in rank:
            if symbol in "12345678":
                files += int(symbol)
            elif symbol in pieces:
                files += 1
            else:
                fail("fen")
            if files > 8:
                fail("fen")
        if files != 8:
            fail("fen")

    if fields[1] not in {"w", "b"}:
        fail("fen")

    castling = fields[2]
    if castling != "-":
        if any(symbol not in "KQkq" for symbol in castling) or len(set(castling)) != len(castling):
            fail("fen")

    en_passant = fields[3]
    if en_passant != "-":
        expected_rank = "6" if fields[1] == "w" else "3"
        if (
            len(en_passant) != 2
            or en_passant[0] not in "abcdefgh"
            or en_passant[1] != expected_rank
        ):
            fail("fen")

    try:
        halfmove = int(fields[4])
        fullmove = int(fields[5])
    except ValueError:
        fail("fen")
    if halfmove < 0 or fullmove <= 0:
        fail("fen")


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


def validate_analysis(analysis, game_fen):
    if not isinstance(analysis, dict):
        fail("analysis")
    if analysis.get("version") != 1 or not isinstance(analysis.get("nodes"), list):
        fail("analysis")

    node_list = analysis["nodes"]
    if not node_list:
        fail("analysis")
    ids = [node.get("id") for node in node_list if isinstance(node, dict)]
    if len(ids) != len(node_list) or any(not isinstance(node_id, str) or not node_id for node_id in ids):
        fail("node-id")
    if len(set(ids)) != len(ids):
        fail("duplicate-node-id")
    nodes = {node["id"]: node for node in node_list}

    root_id = analysis.get("root")
    if not isinstance(root_id, str) or not root_id or root_id not in nodes:
        fail("root")
    root = nodes[root_id]
    if root.get("fen") != game_fen:
        fail("root-fen")
    if any(field in root for field in ("parent", "move", "role", "nags")):
        fail("root")

    for node_id, node in nodes.items():
        if not isinstance(node, dict):
            fail("node-id")
        validate_fen(node.get("fen"))
        children = node.get("children", [])
        if (
            not isinstance(children, list)
            or any(not isinstance(child, str) or not child for child in children)
        ):
            fail("children")
        if len(set(children)) != len(children):
            fail("children")
        for child in children:
            if child not in nodes:
                fail("missing-child")
            if child == root_id:
                fail("root")

        if "comment" in node and not isinstance(node["comment"], str):
            fail("comment")
        if "content" in node:
            validate_spans(node["content"], nodes, root_id)

        if node_id == root_id:
            continue

        parent = node.get("parent")
        if not isinstance(parent, str) or not parent or parent not in nodes:
            fail("parent")

        move = node.get("move")
        if not isinstance(move, dict) or set(move) != {"uci", "san"}:
            fail("move")
        if (
            not isinstance(move["uci"], str)
            or not UCI_RE.fullmatch(move["uci"])
            or move["uci"][:2] == move["uci"][2:4]
        ):
            fail("uci")
        if not isinstance(move["san"], str) or not move["san"]:
            fail("move")

        if normalized_role(node) not in {"main", "alternative", "sideline"}:
            fail("role")
        if "nags" in node:
            nags = node["nags"]
            if not isinstance(nags, list) or any(type(nag) is not int or nag < 0 for nag in nags):
                fail("nags")

    visiting = set()
    visited = set()

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

    projected = []
    main_ids = set()
    cursor = root
    while True:
        main_children = [
            nodes[child_id]
            for child_id in cursor.get("children", [])
            if normalized_role(nodes[child_id]) == "main"
        ]
        if len(main_children) > 1:
            fail("main-line")
        if not main_children:
            break
        cursor = main_children[0]
        main_ids.add(cursor["id"])
        projected.append(cursor["move"]["uci"])

    if not projected:
        fail("empty-main-line")
    if any(
        normalized_role(node) == "main" and node_id not in main_ids
        for node_id, node in nodes.items()
        if node_id != root_id
    ):
        fail("main-line")
    return projected


def validate_review_collection(collection, filename="games.json", encoded_size=None):
    if not is_review_collection_filename(filename):
        fail("filename")
    if encoded_size is not None and encoded_size > MAX_REVIEW_BYTES:
        fail("size")
    if not isinstance(collection, dict) or collection.get("version") != 1:
        fail("collection")
    if "title" in collection and not isinstance(collection["title"], str):
        fail("metadata")
    if "source" in collection and not isinstance(collection["source"], str):
        fail("metadata")

    games = collection.get("games")
    if not isinstance(games, list) or not games:
        fail("collection")

    seen_ids = set()
    for game in games:
        if not isinstance(game, dict):
            fail("game")
        game_id = game.get("id")
        if not isinstance(game_id, str) or not game_id.strip():
            fail("game-id")
        if game_id in seen_ids:
            fail("duplicate-game-id")
        seen_ids.add(game_id)

        if "solution" in game:
            fail("solution")

        fen = game.get("fen")
        validate_fen(fen)

        for field in ("white", "black", "result", "event", "site", "date", "round"):
            value = game.get(field)
            if not isinstance(value, str) or not value.strip():
                fail("metadata")
        if game["result"] not in RESULTS:
            fail("metadata")
        if "source" in game and not isinstance(game["source"], str):
            fail("metadata")

        validate_analysis(game.get("analysis"), fen)


class GameReviewContractTests(unittest.TestCase):
    def test_valid_fixtures_cover_required_contract_features(self):
        standard = json.loads((FIXTURES / "valid-standard.json").read_text(encoding="utf-8"))
        custom = json.loads((FIXTURES / "valid-custom-fen.json").read_text(encoding="utf-8"))
        validate_review_collection(standard, "games-classics.json")
        validate_review_collection(custom, "games.json")

        standard_game = standard["games"][0]
        nodes = standard_game["analysis"]["nodes"]
        self.assertEqual(standard_game["fen"], STANDARD_FEN)
        self.assertIn("José", standard_game["white"])
        self.assertIn("Александр", standard_game["black"])
        self.assertTrue(any(node.get("move", {}).get("uci") == "e1g1" for node in nodes))
        self.assertTrue(any(node.get("comment") for node in nodes))
        self.assertTrue(any(node.get("nags") for node in nodes))
        self.assertTrue(any(node.get("content") for node in nodes))
        self.assertTrue(any(node.get("role") == "alternative" for node in nodes))
        self.assertTrue(
            any(
                node.get("parent") == "v1" and normalized_role(node) == "sideline"
                for node in nodes
            )
        )
        self.assertEqual(
            validate_analysis(standard_game["analysis"], standard_game["fen"]),
            ["e2e4", "e7e5", "g1f3", "b8c6", "f1b5", "g8f6", "e1g1"],
        )

        custom_game = custom["games"][0]
        self.assertNotEqual(custom_game["fen"], STANDARD_FEN)
        self.assertEqual(custom_game["fen"].split()[1], "b")
        self.assertTrue(
            any(len(node.get("move", {}).get("uci", "")) == 5 for node in custom_game["analysis"]["nodes"])
        )
        self.assertNotIn("solution", custom_game)

    def test_invalid_fixtures_fail_for_the_named_contract_reason(self):
        for filename, expected_code in INVALID_FIXTURES.items():
            with self.subTest(filename=filename):
                fixture = json.loads((FIXTURES / filename).read_text(encoding="utf-8"))
                with self.assertRaises(ContractError) as raised:
                    validate_review_collection(fixture)
                self.assertEqual(raised.exception.code, expected_code)

    def test_filename_convention_matches_planned_game_library(self):
        for name in ("games.json", "games-my-games.json", "games-a.b.json"):
            with self.subTest(valid=name):
                self.assertTrue(is_review_collection_filename(name))
        for name in ("games-.json", "game.json", "my-games.json", "games.classics.json", "games.json.bak", "games-a/b.json"):
            with self.subTest(invalid=name):
                self.assertFalse(is_review_collection_filename(name))

    def test_eight_mib_raw_file_cap_is_frozen(self):
        fixture_path = FIXTURES / "valid-standard.json"
        raw = fixture_path.read_bytes()
        self.assertLess(len(raw), MAX_REVIEW_BYTES)
        fixture = json.loads(raw.decode("utf-8"))
        validate_review_collection(fixture, encoded_size=len(raw))
        with self.assertRaises(ContractError) as raised:
            validate_review_collection(fixture, encoded_size=MAX_REVIEW_BYTES + 1)
        self.assertEqual(raised.exception.code, "size")

    def test_format_doc_freezes_review_specific_semantics(self):
        doc = FORMAT_DOC.read_text(encoding="utf-8")
        self.assertIn("games.json", doc)
        self.assertIn("games-<name>.json", doc)
        self.assertIn("8 MiB (8,388,608 bytes)", doc)
        self.assertIn("must not contain `solution`", doc)
        self.assertIn("SetUp", doc)
        self.assertIn("at least one `main` move", doc)
        self.assertIn("analysis.version = 1", doc)

    def test_existing_puzzle_contract_fixtures_remain_distinct_and_untouched(self):
        legacy = json.loads(LEGACY_FIXTURE.read_text(encoding="utf-8"))
        rich = json.loads(RICH_FIXTURE.read_text(encoding="utf-8"))
        self.assertEqual(legacy["version"], 1)
        self.assertEqual(rich["version"], 1)
        self.assertIn("puzzles", legacy)
        self.assertIn("puzzles", rich)
        self.assertNotIn("games", legacy)
        self.assertNotIn("games", rich)


if __name__ == "__main__":
    unittest.main()

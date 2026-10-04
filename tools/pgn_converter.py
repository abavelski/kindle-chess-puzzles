#!/usr/bin/env python3
"""Convert author-authored PGN puzzles into deterministic rich-analysis JSON."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import io
import json
from pathlib import Path
import re
import sys
from typing import Any

try:
    from tools.collection_update import (
        CollectionUpdateError,
        PHASE_TWO_MAX_BYTES,
        atomic_write_bytes,
        compare_collections,
        comparison_messages,
        ensure_no_duplicate_ids,
        load_collection_file,
    )
except ModuleNotFoundError:  # Support direct execution as tools/pgn_converter.py.
    from collection_update import (
        CollectionUpdateError,
        PHASE_TWO_MAX_BYTES,
        atomic_write_bytes,
        compare_collections,
        comparison_messages,
        ensure_no_duplicate_ids,
        load_collection_file,
    )

try:
    import chess
    import chess.pgn
except ImportError as exc:  # pragma: no cover - exercised by direct CLI use without deps.
    raise SystemExit(
        "pgn_converter requires the host-only python-chess dependency (PyPI: chess); "
        "install requirements-tools.txt"
    ) from exc


LEGACY_WARNING_BYTES = 256 * 1024
MOVE_REF_TOKEN = "[%move_ref"
ROLE_ALTERNATIVE_DIRECTIVE = "[%role alternative]"
UCI_RE = re.compile(r"^[a-h][1-8][a-h][1-8][qrbn]?$")


class ConversionError(ValueError):
    """A PGN cannot be converted without producing partial or ambiguous output."""


class _QuietGameBuilder(chess.pgn.GameBuilder):
    def handle_error(self, error: Exception) -> None:
        self.game.errors.append(error)


@dataclass(frozen=True)
class ConversionOptions:
    allow_generated_ids: bool = False
    description_tag: str = "Description"
    difficulty_tag: str = "Difficulty"
    source_tag: str = "Source"
    default_description: str | None = None
    default_difficulty: str | None = None
    default_source: str | None = None
    title: str | None = None
    revision: int | None = None
    collection_source: str | None = None
    compact: bool = False


def _fail(context: str, message: str) -> None:
    raise ConversionError(f"{context}: {message}")


def _nonempty(value: str | None) -> str | None:
    if value is None:
        return None
    value = value.strip()
    return value or None


def _board_fen(board: chess.Board) -> str:
    # Use strict FEN en-passant fields so generated positions are complete standard FEN.
    return board.fen(en_passant="fen")


def _generated_id(game: chess.pgn.Game, start_fen: str, game_index: int) -> str:
    try:
        moves = " ".join(move.uci() for move in game.mainline_moves())
    except Exception as exc:  # pragma: no cover - defensive around malformed parser state.
        _fail(f"game #{game_index}", f"cannot derive generated PuzzleId: {exc}")
    digest = hashlib.sha256(f"{start_fen}\n{moves}".encode("utf-8")).hexdigest()[:16]
    return f"generated-{digest}"


def _puzzle_id(
    game: chess.pgn.Game,
    start_fen: str,
    game_index: int,
    options: ConversionOptions,
) -> str:
    puzzle_id = _nonempty(game.headers.get("PuzzleId"))
    if puzzle_id is not None:
        return puzzle_id
    if options.allow_generated_ids:
        return _generated_id(game, start_fen, game_index)
    _fail(
        f"game #{game_index}",
        "missing required PuzzleId tag (use --allow-generated-ids for explicit fallback IDs)",
    )
    raise AssertionError("unreachable")


def _strip_alternative_directive(raw_comment: str, context: str, branch_root: bool) -> tuple[str, bool]:
    count = raw_comment.count(ROLE_ALTERNATIVE_DIRECTIVE)
    if count == 0:
        return raw_comment, False
    if count > 1:
        _fail(context, "alternative role directive appears more than once")
    if not branch_root:
        _fail(context, "[%role alternative] is only valid on the first move of a non-main variation")

    before, after = raw_comment.split(ROLE_ALTERNATIVE_DIRECTIVE, 1)
    before = before.rstrip()
    after = after.lstrip()
    if before and after:
        visible = f"{before} {after}"
    else:
        visible = before or after
    return visible, True


def _validate_uci_segment(segment: str, context: str) -> None:
    if not UCI_RE.fullmatch(segment) or segment[:2] == segment[2:4]:
        _fail(context, f"invalid move_ref UCI path segment {segment!r}")


def _resolve_path(
    path: str,
    nodes_by_id: dict[str, dict[str, Any]],
    root_id: str,
    context: str,
) -> dict[str, Any]:
    segments = path.split("/") if path else []
    if not segments or any(not segment for segment in segments):
        _fail(context, "move_ref path must contain at least one UCI move")

    current = nodes_by_id[root_id]
    walked: list[str] = []
    for segment in segments:
        _validate_uci_segment(segment, context)
        walked.append(segment)
        matches = []
        for child_id in current.get("children", []):
            child = nodes_by_id[child_id]
            if child["move"]["uci"] == segment:
                matches.append(child)
        if not matches:
            _fail(context, f"dangling move_ref path {'/'.join(walked)!r}")
        if len(matches) > 1:
            _fail(context, f"ambiguous move_ref path {'/'.join(walked)!r}")
        current = matches[0]
    return current


def _structured_text(
    text: str,
    nodes_by_id: dict[str, dict[str, Any]],
    root_id: str,
    context: str,
) -> tuple[list[dict[str, str]], str]:
    """Convert only explicit move_ref directives, preserving all ordinary prose verbatim."""

    if not text:
        return [], ""

    spans: list[dict[str, str]] = []
    projection: list[str] = []
    cursor = 0

    def append_text(value: str) -> None:
        if value:
            spans.append({"type": "text", "text": value})
            projection.append(value)

    while True:
        start = text.find(MOVE_REF_TOKEN, cursor)
        if start < 0:
            append_text(text[cursor:])
            break

        append_text(text[cursor:start])
        token_end = start + len(MOVE_REF_TOKEN)
        if token_end >= len(text) or text[token_end] != " ":
            _fail(context, "malformed move_ref directive; expected a space before the UCI path")

        payload_start = token_end + 1
        close = text.find("]", payload_start)
        if close < 0:
            _fail(context, "unterminated move_ref directive")
        payload = text[payload_start:close]
        if "\n" in payload or "\r" in payload:
            _fail(context, "move_ref directive cannot contain a newline")

        if "|" in payload:
            path_raw, label_raw = payload.split("|", 1)
            label = label_raw.strip()
            if not label:
                _fail(context, "move_ref label must be non-empty")
        else:
            path_raw = payload
            label = None

        path = path_raw.strip()
        target = _resolve_path(path, nodes_by_id, root_id, context)
        span: dict[str, str] = {"type": "move_ref", "node": target["id"]}
        if label is not None:
            span["label"] = label
        spans.append(span)
        projection.append(label if label is not None else target["move"]["san"])
        cursor = close + 1

    return spans, "".join(projection)


def _build_tree(game: chess.pgn.Game, puzzle_id: str, start_board: chess.Board) -> tuple[list[dict[str, Any]], dict[str, str]]:
    root = {"id": "n0", "fen": _board_fen(start_board)}
    nodes: list[dict[str, Any]] = [root]
    raw_comments: dict[str, str] = {"n0": game.comment or ""}
    next_id = 1

    if ROLE_ALTERNATIVE_DIRECTIVE in raw_comments["n0"]:
        _fail(f"puzzle {puzzle_id} root comment", "[%role alternative] cannot be attached to the root")

    def create_child(
        child: chess.pgn.ChildNode,
        json_parent: dict[str, Any],
        board: chess.Board,
        *,
        branch_root: bool,
        on_main_path: bool,
    ) -> tuple[dict[str, Any], chess.Board]:
        nonlocal next_id
        context = f"puzzle {puzzle_id} move {next_id}"
        move = child.move
        if move not in board.legal_moves:
            _fail(context, f"illegal move {move.uci()} for position {board.fen(en_passant='fen')}")

        try:
            san = board.san(move)
        except Exception as exc:
            _fail(context, f"cannot compute SAN for {move.uci()}: {exc}")

        child_board = board.copy(stack=False)
        child_board.push(move)
        node_id = f"n{next_id}"
        next_id += 1

        raw_parts = [part for part in (getattr(child, "starting_comment", ""), child.comment or "") if part]
        raw_comment, is_alternative = _strip_alternative_directive(
            "\n".join(raw_parts),
            f"puzzle {puzzle_id} comment on {node_id}",
            branch_root=branch_root,
        )
        role = "main" if on_main_path else ("alternative" if is_alternative else "sideline")

        node: dict[str, Any] = {
            "id": node_id,
            "parent": json_parent["id"],
            "move": {"uci": move.uci(), "san": san},
            "fen": _board_fen(child_board),
            "role": role,
        }
        nags = sorted(int(nag) for nag in child.nags)
        if nags:
            node["nags"] = nags

        json_parent.setdefault("children", []).append(node_id)
        nodes.append(node)
        raw_comments[node_id] = raw_comment
        return node, child_board

    def visit_position(
        pgn_parent: chess.pgn.GameNode,
        json_parent: dict[str, Any],
        board: chess.Board,
        parent_on_main_path: bool,
    ) -> None:
        if not pgn_parent.variations:
            return

        # PGN source order emits the main move, then all RAVs attached to that
        # position, then continues the main line. Mirror that order in flat IDs.
        main_child = pgn_parent.variations[0]
        main_json, main_board = create_child(
            main_child,
            json_parent,
            board,
            branch_root=False,
            on_main_path=parent_on_main_path,
        )

        for side_child in pgn_parent.variations[1:]:
            side_json, side_board = create_child(
                side_child,
                json_parent,
                board,
                branch_root=True,
                on_main_path=False,
            )
            visit_position(side_child, side_json, side_board, False)

        visit_position(main_child, main_json, main_board, parent_on_main_path)

    visit_position(game, root, start_board, True)
    return nodes, raw_comments


def _metadata_value(headers: chess.pgn.Headers, tag: str, default: str | None) -> str | None:
    value = _nonempty(headers.get(tag))
    if value is not None:
        return value
    return _nonempty(default)


def _convert_game(game: chess.pgn.Game, game_index: int, options: ConversionOptions) -> dict[str, Any]:
    if game.errors:
        header_id = _nonempty(game.headers.get("PuzzleId"))
        context = f"puzzle {header_id}" if header_id else f"game #{game_index}"
        _fail(context, f"malformed or illegal PGN: {game.errors[0]}")

    try:
        start_board = game.board()
    except Exception as exc:
        header_id = _nonempty(game.headers.get("PuzzleId"))
        context = f"puzzle {header_id}" if header_id else f"game #{game_index}"
        _fail(context, f"invalid starting FEN/SetUp: {exc}")

    if not start_board.is_valid():
        header_id = _nonempty(game.headers.get("PuzzleId"))
        context = f"puzzle {header_id}" if header_id else f"game #{game_index}"
        _fail(context, f"invalid starting chess position: status={start_board.status()}")

    start_fen = _board_fen(start_board)
    puzzle_id = _puzzle_id(game, start_fen, game_index, options)
    nodes, raw_comments = _build_tree(game, puzzle_id, start_board)
    if len(nodes) == 1:
        _fail(f"puzzle {puzzle_id}", "PGN contains no moves")

    nodes_by_id = {node["id"]: node for node in nodes}
    root_id = "n0"

    for node in nodes:
        raw = raw_comments[node["id"]]
        if not raw:
            continue
        spans, projection = _structured_text(
            raw,
            nodes_by_id,
            root_id,
            f"puzzle {puzzle_id} comment on {node['id']}",
        )
        node["comment"] = projection
        node["content"] = spans

    solution: list[str] = []
    cursor = nodes_by_id[root_id]
    while True:
        main_children = [
            nodes_by_id[child_id]
            for child_id in cursor.get("children", [])
            if nodes_by_id[child_id].get("role") == "main"
        ]
        if len(main_children) > 1:  # pragma: no cover - construction prevents this.
            _fail(f"puzzle {puzzle_id}", "multiple main children in generated analysis")
        if not main_children:
            break
        cursor = main_children[0]
        solution.append(cursor["move"]["uci"])

    if not solution:
        _fail(f"puzzle {puzzle_id}", "main line contains no moves")

    puzzle: dict[str, Any] = {
        "id": puzzle_id,
        "fen": start_fen,
    }

    description = _metadata_value(game.headers, options.description_tag, options.default_description)
    if description is not None:
        spans, projection = _structured_text(
            description,
            nodes_by_id,
            root_id,
            f"puzzle {puzzle_id} description",
        )
        puzzle["description"] = projection
        puzzle["description_content"] = spans

    difficulty = _metadata_value(game.headers, options.difficulty_tag, options.default_difficulty)
    if difficulty is not None:
        puzzle["difficulty"] = difficulty

    source = _metadata_value(game.headers, options.source_tag, options.default_source)
    if source is not None:
        puzzle["source"] = source

    puzzle["solution"] = solution
    puzzle["analysis"] = {"version": 1, "root": root_id, "nodes": nodes}
    return puzzle


def convert_pgn_text(text: str, options: ConversionOptions | None = None) -> dict[str, Any]:
    options = options or ConversionOptions()
    stream = io.StringIO(text)
    puzzles: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    game_index = 0

    while True:
        try:
            game = chess.pgn.read_game(stream, Visitor=_QuietGameBuilder)
        except Exception as exc:
            _fail(f"game #{game_index + 1}", f"malformed PGN: {exc}")
        if game is None:
            break
        game_index += 1
        puzzle = _convert_game(game, game_index, options)
        if puzzle["id"] in seen_ids:
            _fail(f"puzzle {puzzle['id']}", "duplicate PuzzleId in input")
        seen_ids.add(puzzle["id"])
        puzzles.append(puzzle)

    if not puzzles:
        _fail("input", "PGN contains no games")

    collection: dict[str, Any] = {"version": 1}
    if options.revision is not None:
        collection["revision"] = options.revision
    if _nonempty(options.title) is not None:
        collection["title"] = _nonempty(options.title)
    if _nonempty(options.collection_source) is not None:
        collection["source"] = _nonempty(options.collection_source)
    collection["puzzles"] = puzzles
    return collection


def encode_collection(collection: dict[str, Any], *, compact: bool = False) -> bytes:
    if compact:
        text = json.dumps(collection, ensure_ascii=False, separators=(",", ":")) + "\n"
    else:
        text = json.dumps(collection, ensure_ascii=False, indent=2) + "\n"
    return text.encode("utf-8")


def size_messages(encoded_size: int) -> list[str]:
    messages = [
        f"encoded size: {encoded_size} bytes",
        (
            "size thresholds: legacy 256 KiB (262144 bytes); "
            f"phase-two 8 MiB ({PHASE_TWO_MAX_BYTES} bytes)"
        ),
    ]
    if encoded_size > LEGACY_WARNING_BYTES:
        messages.append(
            "warning: output exceeds 256 KiB (262144 bytes); phase-one/legacy readers may reject it"
        )
    if encoded_size > PHASE_TWO_MAX_BYTES:
        messages.append(
            "warning: output exceeds phase-two 8 MiB (8388608 bytes); split the collection"
        )
    return messages


def _read_input(path: str) -> str:
    if path == "-":
        return sys.stdin.read()
    return Path(path).read_text(encoding="utf-8")


def _write_output(path: str, data: bytes, collection: dict[str, Any]) -> None:
    if path == "-":
        sys.stdout.buffer.write(data)
        return

    output_path = Path(path)
    if output_path.exists():
        previous = load_collection_file(output_path)
        comparison = compare_collections(previous, collection)
        for message in comparison_messages(comparison):
            print(message, file=sys.stderr)
        ensure_no_duplicate_ids(comparison)

    atomic_write_bytes(output_path, data)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", help="PGN file, or - for stdin")
    parser.add_argument("-o", "--output", default="-", help="JSON file, or - for stdout")
    parser.add_argument("--allow-generated-ids", action="store_true", help="explicitly opt in to deterministic generated IDs when PuzzleId is absent")
    parser.add_argument("--title", help="optional collection title")
    parser.add_argument("--revision", type=int, help="optional collection content revision")
    parser.add_argument("--collection-source", help="optional collection-level source metadata")
    parser.add_argument("--description-tag", default="Description", help="PGN tag used for puzzle description")
    parser.add_argument("--difficulty-tag", default="Difficulty", help="PGN tag used for puzzle difficulty")
    parser.add_argument("--source-tag", default="Source", help="PGN tag used for puzzle source")
    parser.add_argument("--default-description", help="description used only when the configured tag is absent/blank")
    parser.add_argument("--default-difficulty", help="difficulty used only when the configured tag is absent/blank")
    parser.add_argument("--default-source", help="source used only when the configured tag is absent/blank")
    parser.add_argument("--compact", action="store_true", help="emit compact deterministic JSON instead of two-space pretty JSON")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    options = ConversionOptions(
        allow_generated_ids=args.allow_generated_ids,
        description_tag=args.description_tag,
        difficulty_tag=args.difficulty_tag,
        source_tag=args.source_tag,
        default_description=args.default_description,
        default_difficulty=args.default_difficulty,
        default_source=args.default_source,
        title=args.title,
        revision=args.revision,
        collection_source=args.collection_source,
        compact=args.compact,
    )

    try:
        collection = convert_pgn_text(_read_input(args.input), options)
        encoded = encode_collection(collection, compact=options.compact)
        _write_output(args.output, encoded, collection)
    except (ConversionError, CollectionUpdateError, OSError, UnicodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    for message in size_messages(len(encoded)):
        print(message, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Convert ordinary PGN games into deterministic game-review JSON."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import io
import json
from pathlib import Path
import sys
from typing import Any

try:
    from tools import pgn_converter as shared
    from tools.collection_update import CollectionUpdateError, atomic_write_bytes
except ModuleNotFoundError:  # Support direct execution as tools/pgn_review_converter.py.
    import pgn_converter as shared
    from collection_update import CollectionUpdateError, atomic_write_bytes


MAX_REVIEW_BYTES = 8 * 1024 * 1024
RESULTS = {"1-0", "0-1", "1/2-1/2", "*"}
IDENTITY_HEADERS = ("Event", "Site", "Date", "Round", "White", "Black", "Result")
DISPLAY_DEFAULTS = {
    "White": "?",
    "Black": "?",
    "Result": "*",
    "Event": "?",
    "Site": "?",
    "Date": "????.??.??",
    "Round": "?",
}

ConversionError = shared.ConversionError


@dataclass(frozen=True)
class ReviewConversionOptions:
    id_tag: str = "GameId"
    game_id: str | None = None
    source_tag: str = "Source"
    default_source: str | None = None
    title: str | None = None
    collection_source: str | None = None
    compact: bool = False


def _display_header(headers: Any, tag: str) -> str:
    value = shared._nonempty(headers.get(tag))
    if value is not None:
        return value
    return DISPLAY_DEFAULTS[tag]


def _generated_game_id(game: Any, start_fen: str, game_index: int) -> str:
    try:
        moves = [move.uci() for move in game.mainline_moves()]
    except Exception as exc:  # pragma: no cover - defensive around malformed parser state.
        shared._fail(f"game #{game_index}", f"cannot derive generated game ID: {exc}")
    identity = {
        "headers": [[tag, _display_header(game.headers, tag)] for tag in IDENTITY_HEADERS],
        "fen": start_fen,
        "moves": moves,
    }
    canonical = json.dumps(identity, ensure_ascii=False, separators=(",", ":"))
    digest = hashlib.sha256(canonical.encode("utf-8")).hexdigest()[:20]
    return f"game-{digest}"


def _game_id(game: Any, start_fen: str, game_index: int, options: ReviewConversionOptions) -> str:
    cli_id = shared._nonempty(options.game_id)
    if cli_id is not None:
        return cli_id
    tagged = shared._nonempty(game.headers.get(options.id_tag))
    if tagged is not None:
        return tagged
    return _generated_game_id(game, start_fen, game_index)


def _game_context(game: Any, game_index: int, options: ReviewConversionOptions) -> str:
    explicit = shared._nonempty(options.game_id) or shared._nonempty(game.headers.get(options.id_tag))
    return f"game {explicit}" if explicit is not None else f"game #{game_index}"


def _convert_game(game: Any, game_index: int, options: ReviewConversionOptions) -> dict[str, Any]:
    context = _game_context(game, game_index, options)
    if game.errors:
        shared._fail(context, f"malformed or illegal PGN: {game.errors[0]}")

    try:
        start_board = game.board()
    except Exception as exc:
        shared._fail(context, f"invalid starting FEN/SetUp: {exc}")
    if not start_board.is_valid():
        shared._fail(context, f"invalid starting chess position: status={start_board.status()}")

    start_fen = shared._board_fen(start_board)
    game_id = _game_id(game, start_fen, game_index, options)
    nodes, raw_comments = shared._build_tree(
        game,
        game_id,
        start_board,
        context_kind="game",
    )
    if len(nodes) == 1:
        shared._fail(f"game {game_id}", "PGN contains no moves")

    nodes_by_id = {node["id"]: node for node in nodes}
    root_id = "n0"
    for node in nodes:
        raw = raw_comments[node["id"]]
        if not raw:
            continue
        spans, projection = shared._structured_text(
            raw,
            nodes_by_id,
            root_id,
            f"game {game_id} comment on {node['id']}",
        )
        node["comment"] = projection
        node["content"] = spans

    result = _display_header(game.headers, "Result")
    if result not in RESULTS:
        shared._fail(f"game {game_id}", f"unsupported Result header {result!r}")

    review_game: dict[str, Any] = {
        "id": game_id,
        "fen": start_fen,
        "white": _display_header(game.headers, "White"),
        "black": _display_header(game.headers, "Black"),
        "result": result,
        "event": _display_header(game.headers, "Event"),
        "site": _display_header(game.headers, "Site"),
        "date": _display_header(game.headers, "Date"),
        "round": _display_header(game.headers, "Round"),
    }
    source = shared._metadata_value(game.headers, options.source_tag, options.default_source)
    if source is not None:
        review_game["source"] = source
    review_game["analysis"] = {"version": 1, "root": root_id, "nodes": nodes}
    return review_game


def convert_pgn_text(text: str, options: ReviewConversionOptions | None = None) -> dict[str, Any]:
    options = options or ReviewConversionOptions()
    stream = io.StringIO(text)
    games: list[dict[str, Any]] = []
    seen_ids: dict[str, int] = {}
    game_index = 0

    while True:
        try:
            game = shared.chess.pgn.read_game(stream, Visitor=shared._QuietGameBuilder)
        except Exception as exc:
            shared._fail(f"game #{game_index + 1}", f"malformed PGN: {exc}")
        if game is None:
            break
        game_index += 1
        if options.game_id is not None and game_index > 1:
            shared._fail("input", "--game-id can only be used with a single-game PGN")
        review_game = _convert_game(game, game_index, options)
        previous = seen_ids.get(review_game["id"])
        if previous is not None:
            shared._fail(
                f"game #{game_index} (id {review_game['id']})",
                f"duplicate game ID {review_game['id']!r}; first used by game #{previous}",
            )
        seen_ids[review_game["id"]] = game_index
        games.append(review_game)

    if not games:
        shared._fail("input", "PGN contains no games")

    collection: dict[str, Any] = {"version": 1}
    title = shared._nonempty(options.title)
    if title is not None:
        collection["title"] = title
    collection_source = shared._nonempty(options.collection_source)
    if collection_source is not None:
        collection["source"] = collection_source
    collection["games"] = games
    return collection


def encode_collection(collection: dict[str, Any], *, compact: bool = False) -> bytes:
    if compact:
        text = json.dumps(collection, ensure_ascii=False, separators=(",", ":")) + "\n"
    else:
        text = json.dumps(collection, ensure_ascii=False, indent=2) + "\n"
    return text.encode("utf-8")


def validate_encoded_size(encoded_size: int) -> None:
    if encoded_size > MAX_REVIEW_BYTES:
        shared._fail(
            "output",
            f"encoded size {encoded_size} bytes exceeds review limit {MAX_REVIEW_BYTES} bytes (8 MiB)",
        )


def size_messages(encoded_size: int) -> list[str]:
    return [
        f"encoded size: {encoded_size} bytes",
        f"review size limit: 8 MiB ({MAX_REVIEW_BYTES} bytes)",
    ]


def _read_input(path: str) -> str:
    if path == "-":
        return sys.stdin.read()
    return Path(path).read_text(encoding="utf-8")


def _write_output(path: str, data: bytes) -> None:
    if path == "-":
        sys.stdout.buffer.write(data)
        return
    atomic_write_bytes(Path(path), data)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", help="PGN file, or - for stdin")
    parser.add_argument("-o", "--output", default="-", help="review JSON file, or - for stdout")
    parser.add_argument("--id-tag", default="GameId", help="PGN tag used as an explicit game ID override")
    parser.add_argument("--game-id", help="explicit ID override for a single-game PGN")
    parser.add_argument("--title", help="optional collection title")
    parser.add_argument("--collection-source", help="optional collection-level source metadata")
    parser.add_argument("--source-tag", default="Source", help="PGN tag used for per-game source metadata")
    parser.add_argument("--default-source", help="per-game source used when the configured source tag is absent/blank")
    parser.add_argument("--compact", action="store_true", help="emit compact deterministic JSON instead of two-space pretty JSON")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    options = ReviewConversionOptions(
        id_tag=args.id_tag,
        game_id=args.game_id,
        source_tag=args.source_tag,
        default_source=args.default_source,
        title=args.title,
        collection_source=args.collection_source,
        compact=args.compact,
    )
    try:
        collection = convert_pgn_text(_read_input(args.input), options)
        encoded = encode_collection(collection, compact=options.compact)
        validate_encoded_size(len(encoded))
        _write_output(args.output, encoded)
    except (ConversionError, CollectionUpdateError, OSError, UnicodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    for message in size_messages(len(encoded)):
        print(message, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

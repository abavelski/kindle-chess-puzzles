#!/usr/bin/env python3
"""Remove redundant imported book description projections from local JSON files."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

try:
    from .collection_update import (
        atomic_write_bytes, compare_collections, ensure_no_duplicate_ids, load_collection_file,
    )
except ImportError:
    from collection_update import (
        atomic_write_bytes, compare_collections, ensure_no_duplicate_ids, load_collection_file,
    )


def main_path(analysis: dict) -> list[str]:
    nodes = {node["id"]: node for node in analysis.get("nodes", [])}
    current = nodes.get(analysis.get("root"), {})
    path = []
    while current:
        children = [nodes[key] for key in current.get("children", []) if key in nodes]
        children = [node for node in children if node.get("role") == "main"]
        if not children:
            return path
        if len(children) != 1 or children[0]["id"] in path:
            return []
        current = children[0]
        path.append(current["id"])
    return []


def normalized(text: str) -> str:
    return "".join(character.casefold() for character in text if character.isalnum())


def rendered_comments(analysis: dict) -> str:
    return " ".join(
        "".join(span.get("text", "") for span in node.get("content", [
            {"type": "text", "text": node.get("comment", "")},
        ]) if span.get("type") == "text")
        for node in analysis.get("nodes", [])
    )


def clean_collection(collection: dict) -> tuple[dict, int]:
    cleaned = copy.deepcopy(collection)
    changed = 0
    for puzzle in cleaned["puzzles"]:
        analysis = puzzle.get("analysis")
        spans = puzzle.get("description_content")
        if not isinstance(analysis, dict) or not isinstance(spans, list):
            continue
        path = main_path(analysis)
        references = [span.get("node") for span in spans if span.get("type") == "move_ref"]
        if not path or references != path or any(span.get("type") not in ("text", "move_ref") for span in spans):
            continue
        text = "".join(span["text"] for span in spans if span["type"] == "text").strip()
        reference = puzzle.get("reference") or ""
        if text and (not reference or text != f"Reference: {reference}"):
            continue
        pgn = puzzle.get("pgn")
        if pgn is not None:
            # Only clear the known imported plain projection; authored prose stays intact.
            body = pgn.partition("\n\n")[2].strip()
            expected = body + (f"\nReference: {reference}" if reference else "")
            if not body or puzzle.get("description") != expected:
                continue
        reference_key = normalized(reference)
        comments = rendered_comments(analysis)
        duplicate_reference = (reference_key in normalized(comments) if reference_key
                               else bool(reference) and reference in comments)
        remaining = text if text and not duplicate_reference else ""
        puzzle["description_content"] = [{"type": "text", "text": remaining}] if remaining else []
        if pgn is not None:
            puzzle["description"] = remaining
        # Fallback puzzles have original notes without PGN; preserve those for NOTE.
        changed += 1
    return cleaned, changed


def clean_file(path: Path) -> int:
    previous = load_collection_file(path)
    cleaned, changed = clean_collection(previous)
    ensure_no_duplicate_ids(compare_collections(previous, cleaned))
    if changed:
        atomic_write_bytes(path, (json.dumps(cleaned, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    return changed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("collections", type=Path, nargs="+", help="local collection JSON files")
    args = parser.parse_args()
    for path in args.collections:
        print(f"{path}: cleaned {clean_file(path)} redundant analysis descriptions")


if __name__ == "__main__":
    main()

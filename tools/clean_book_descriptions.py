#!/usr/bin/env python3
"""Remove verified imported book metadata headings from local puzzle collections."""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
import re

try:
    from .collection_update import (
        atomic_write_bytes, compare_collections, ensure_no_duplicate_ids, load_collection_file,
    )
except ImportError:  # Direct script invocation.
    from collection_update import (
        atomic_write_bytes, compare_collections, ensure_no_duplicate_ids, load_collection_file,
    )


def clean_collection(collection: dict) -> tuple[dict, int]:
    cleaned = copy.deepcopy(collection)
    changed = 0
    for puzzle in cleaned["puzzles"]:
        identity = re.fullmatch(r"([0-9]+)([wb])", puzzle.get("id", ""))
        if identity is None or puzzle.get("difficulty") is None:
            continue
        number, color = identity.groups()
        fen_fields = puzzle.get("fen", "").split()
        if len(fen_fields) != 6 or fen_fields[1] != color:
            continue
        side = "white" if color == "w" else "black"
        heading = f"{number} - {side} - {puzzle['difficulty']}"
        edited = False
        description = puzzle.get("description")
        if isinstance(description, str) and description.partition("\n")[0] == heading:
            puzzle["description"] = description.partition("\n")[2]
            edited = True
        spans = puzzle.get("description_content")
        if isinstance(spans, list) and spans and spans[0].get("type") == "text":
            first = spans[0]["text"]
            if first.partition("\n")[0] == heading:
                remaining = first.partition("\n")[2]
                if remaining:
                    spans[0]["text"] = remaining
                else:
                    spans.pop(0)
                edited = True
        changed += edited
    return cleaned, changed


def clean_file(path: Path) -> int:
    previous = load_collection_file(path)
    cleaned, changed = clean_collection(previous)
    comparison = compare_collections(previous, cleaned)
    ensure_no_duplicate_ids(comparison)
    if changed:
        encoded = (json.dumps(cleaned, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
        atomic_write_bytes(path, encoded)
    return changed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("collections", type=Path, nargs="+", help="local collection JSON files to update")
    args = parser.parse_args()
    for path in args.collections:
        print(f"{path}: cleaned {clean_file(path)} puzzle descriptions")


if __name__ == "__main__":
    main()

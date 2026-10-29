#!/usr/bin/env python3
"""Unwrap pre prose and remove unsupported csl/cal board marks from PGN comments."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

try:
    from tools.collection_update import CollectionUpdateError, atomic_write_bytes
except ModuleNotFoundError:
    from collection_update import CollectionUpdateError, atomic_write_bytes


def clean_text(text: str) -> tuple[str, int]:
    changed = 0

    def clean_comment(comment: str) -> str:
        nonlocal changed
        pattern = r'\[%pre\s+([^\]]*)\]'
        cleaned, count = re.subn(pattern, lambda match: match[1], comment)
        if re.search(r'\[%pre(?:\s|\])', cleaned):
            raise ValueError('Unclosed or empty [%pre ...] wrapper in PGN comment')
        cleaned, removed = re.subn(r'\[%(?:csl|cal)(?:\s+[^\]]*)?\]', '', cleaned)
        if re.search(r'\[%(?:csl|cal)(?:\s|\])', cleaned):
            raise ValueError('Unclosed [%csl ...] or [%cal ...] directive in PGN comment')
        changed += count + removed
        return cleaned

    # Quoted tag values are consumed unchanged; only brace/line comments are edited.
    tokens = r'"(?:\\.|[^"\\])*"|\{[^}]*\}|;[^\r\n]*'
    cleaned = re.sub(
        tokens,
        lambda match: clean_comment(match[0]) if match[0][0] in '{;' else match[0],
        text,
    )
    return cleaned, changed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path, help='source UTF-8 PGN')
    parser.add_argument('-o', '--output', type=Path, required=True, help='cleaned PGN destination')
    args = parser.parse_args(argv)
    try:
        cleaned, count = clean_text(args.input.read_bytes().decode('utf-8'))
        atomic_write_bytes(args.output, cleaned.encode('utf-8'))
    except (OSError, UnicodeError, ValueError, CollectionUpdateError) as exc:
        print(f'error: {exc}', file=sys.stderr)
        return 2
    print(f'{args.output}: cleaned {count} directives (pre wrappers and csl/cal marks)')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

# Task 32 — Remove redundant solution headings

**Status:** Implemented
**Depends on:** completed Tasks 30–31
**Validation:** host automated tests and reviewed Gray8 snapshots; no physical checkpoint
**Working branch:** main (post-phase-two improvement)

Remove the imported `number - side - difficulty` first line from both plain
descriptions and structured description spans in the four local book collections.
Keep filenames, puzzle IDs/order, FENs, difficulty, topics, solutions, analysis
nodes and explicit move references intact. Make the cleanup reproducible with an
offline tool; arbitrary authored descriptions must remain untouched. The runtime
must not rewrite collections or reinterpret version-1 description semantics.

Remove the analysis panel's `ANALYSIS current/total` heading and separator, using
the reclaimed vertical space for solution rows. Retain existing page navigation,
monochrome move affordances and hit testing. Do not change other layout regions.

Acceptance: failing-first tests cover cleanup of plain/structured descriptions,
exact metadata matching, meaningful prose/references preserved, repeat-run
idempotence, atomic replacement and unchanged progress; renderer tests prove
content begins at the panel top and reclaimed rows reduce overflow while page
controls and every move remain reachable. Review changed first/middle/last-page
snapshots. Run scripts/check.sh and scripts/build-kindle.sh. Prepare updated local
collections and binary; do not deploy or access the Kindle for this task.

## Implementation and validation — 2026-10-04

Confirmed the imported heading was stored in both `description` and the first
`description_content` text span in every local book puzzle, including `135b`.
The offline cleanup tool removes an exact heading only after matching numbered
ID, FEN side and difficulty. Both projections lose the heading; substantive
prose, references and all other metadata remain intact. Runtime parsing and
description semantics are unchanged. Repeat cleanup reports zero changes and
does not write files. Original private collection copies remain locally under
`target/task32/original-collections/`.

Renderer regression tests failed first on the old heading offset (44 pixels at
Scribe metrics) and a two-page result for content that should fit in one page.
The Python cleanup tests then failed on the retained headings. All passed after
implementation. The analysis title and separator are removed; body rows start
at the content top, with existing footer controls and move-chip geometry intact.

Reviewed all nine changed analysis Gray8 snapshots, including selected moves,
nested variations and first/middle/last pages, before updating their checksums.
Pixels above the analysis content area were unchanged in all nine comparisons.
Also reviewed real `135b` description/analysis before and after cleanup. The
four updated collections parse successfully through chess-core (1,615 puzzles).
Structural comparisons verified unchanged root metadata, filenames, puzzle
IDs/order, FENs, difficulty, topics, solutions, PGN, references and analysis nodes;
only the two description fields changed. No progress file or Kindle was accessed.

`scripts/build-kindle.sh` passed its complete `scripts/check.sh` gate (fmt,
clippy, workspace tests, renderer snapshots and Python/platform/packaging
contracts) and produced the validated ARMv7/glibc-2.35 release. Host Python tests
used the existing `/tmp/kindle-enrich-venv` with pinned chess 1.11.2.
Binary: `target/kindle/kindle-chess`, SHA-256:
`995dca18e2d4c7e8d71b5253a7d2b58941834ec8f285178a49dd47f4951f6d12`.

Prepared local collection hashes:

| Filename | SHA-256 |
| --- | --- |
| puzzles-eink-book-001.json | 825f62cd7b7e67aa6230b38066f1492812f072e3fad2b8f8d1219a62ad4152e6 |
| puzzles-eink-book-002.json | 68ed5918d02f4572dc12d30991d9f564138f9bf0439be70b7ebfdd96f3c848b8 |
| puzzles-eink-book-003.json | ddacdeee0ebf3b71187294f64c41c7b705a33cd53648882c085f25fb27b6c563 |
| puzzles-eink-book-004.json | 38ff9d4e0afd037f643a5ee4af866e988f96b1a404eafde4c4586e1d4b115a07 |

Deployment was initially deferred at the user's request. Physical readability
of this change has not been tested; automated acceptance requires no device
checkpoint.

## Deployment authorized and completed — 2026-10-04

The user subsequently requested deployment. Staged the matching build receipt
with `scripts/stage-kindle.sh`, then installed with
`scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222` using the documented
empty-password SSH authentication. The app was stopped before and after the update.
The installed binary hash matches the validated release above and the device
dynamic loader resolves its dependencies.

Uploaded the four cleaned local collections to a temporary directory on the
same filesystem. Verified uploaded hashes and the expected original device
hashes before replacing each file by rename, with originals available for
rollback on failure. Verified the installed collections match the four hashes
above; removed temporary upload files after success. The stylus fixture was
unchanged. Progress before/after SHA-256 matches:
`816f4cd5927c30ff9320c14998a1c438d16897bfd19a0b16ffa6bbdcba0c0b60`.
Ready for library launch; physical readability remains pending user review.

# Task 34 — Clean redundant book analysis descriptions

**Status:** Implemented
**Depends on:** completed Task 33
**Working branch:** main (post-phase-two improvement)
**Validation:** host tests and reviewed collection-render previews; no physical checkpoint

Clean the four local imported book JSON collections with a reproducible offline
script. Remove the structured complete-main-path summary from description_content
and Reference lines already present in analysis comments. Clear the matching plain
PGN description projection, retaining unique attribution. Preserve arbitrary
prose, partial/non-main references and original notes for fallback puzzles without
PGN. Do not change renderer behavior or uploaded collections at runtime.

Keep filenames, IDs/order, FENs, topics, difficulty, solutions, PGN/source metadata,
analysis nodes, comments and explicit references in analysis intact. Keep progress
separate and untouched. Start with failing tests for both description projections,
duplicate and unique attribution, preservation, idempotence and atomic updates.
Review real before/after renders; validate cleaned collections through chess-core.
Run full host checks and the Kindle cross-build. Commit/push to main and deploy
only the cleaned JSON through checksum-verified same-filesystem replacements,
with rollback originals. Verify installed collection and unchanged progress hashes.

## Findings — 2026-10-04

All 1,615 local book descriptions contain a complete-main-path summary. All 808
Reference attributions also occur in the tree comments; some combine text from
several comments. Nine fallback puzzles have no PGN and retain original plain
notes/ERROR markers; their structured main-line summaries are still redundant.
The user requested an offline JSON cleanup instead of runtime display filtering.

## Implementation and validation — 2026-10-04

Added tools/clean_book_analysis.py with failing-first synthetic tests for both
projections, unique/punctuation-only attribution, null references, comments split
across nodes, authoritative comment content, authored/partial/non-main content,
original fallback notes, atomic failure recovery and byte-stable repeat runs.
Seven focused Python tests pass, and the new suite is part of scripts/check.sh.

Cleaned all four ignored local collections (500/500/500/115 puzzles). Only
`description` and `description_content` changed. Every structured preface is now
empty; plain PGN duplicates are cleared, with nine original no-PGN notes retained.
A repeat run reports zero changes. Original files are retained locally under
`target/task34/original-collections/`; atomic tool output matches local data exactly.

The unchanged chess-core parser and renderer audited all 1,615 cleaned puzzles
and every analysis page: all PGN tree moves/comment references remain available,
PGN starts at the panel top, and analysis browsing leaves progress unchanged.
Reviewed real 135b before/after render: two redundant rows disappear. The resulting
frame matches the reviewed compact preview; selected/unselected states and all
existing deterministic renderer snapshots pass without changes to the renderer.

The complete scripts/build-kindle.sh gate passed: formatting, clippy, workspace
and snapshot tests, Python/platform/packaging contracts and ARMv7/glibc-2.35 build.
The binary is byte-identical to the already installed Task 33 build
`f63264af763fff428560de70b7e11739d80654a0791599e5b72728a4393a316e`;
this task deploys collection data only. Collection hashes prepared for deployment:

| Filename | SHA-256 |
| --- | --- |
| puzzles-eink-book-001.json | b22b1b1b2bd4b331ee00ce154934b576ce1e31bc59867543c8b833d1e992edf3 |
| puzzles-eink-book-002.json | 8dfadf965472f5e6d744df85e10435c1bd1af887c125020af39f1042ba4bad0a |
| puzzles-eink-book-003.json | b8c0eaed5e01a19c023db746594d3ca024009a5daf09d8bb3e5fe8d0a65e5357 |
| puzzles-eink-book-004.json | 7685d3583e42855c80a760454223e9727623bffa881f18444cfe220e78ed5d6e |


## Deployment — 2026-10-04

Committed cleanup/tests/docs as `a265f53` and pushed to main before deployment.
Uploaded only the four cleaned JSON files and checksum/install metadata through
verified scp-over-SSH at root@192.168.1.20:2222. The app was already stopped;
no lifecycle signal was needed. Temporary installation/app locks prevented
concurrent updates/launch while files were replaced. Verified original device
hashes and uploaded hashes before same-filesystem renames, retaining originals
for rollback until all installed hashes and progress checks passed.

All four installed collection hashes match the table above. The stylus fixture
remains `3ddf457bbac3472c3f952ed082c46904023e644ddd2c814b9404729bfde27b84`.
Progress before/after remains
`61f5aef3786faa56ae8d2396c0447dfb5312272d765787ccff20d648456fd9c6`.
Runtime binary remains the identical Task 33 build. App/locks and temporary
upload files were absent after installation; native Xorg/awesome remained present.
Ready for library launch; physical appearance awaits user review.

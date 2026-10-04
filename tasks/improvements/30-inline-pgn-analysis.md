# Task 30 — Inline PGN analysis movetext

**Status:** Implemented
**Depends on:** completed Tasks 20–25, 28–29
**Validation:** host automated tests and reviewed Gray8 snapshots; no physical checkpoint
**Working branch:** phase-2 (main remains unchanged)

Replace vertically labeled analysis rows with compact inline PGN movetext. Preserve
ordered children: first child continues each line, subsequent children are recursive
parenthesized variations inserted after that first move and before its continuation.
Use validated SAN, FEN side/fullmove numbering, $N NAGs and braced structured comments.
Preserve root/description prose, explicit interactive references, selected styling,
padded unambiguous hit targets, overflow pagination, toolbar close, exact stored-FEN
preview and unchanged durable progress. No schema, converter, device input, grading,
application-state or legacy-puzzle behavior changes.

Acceptance: notation assertions cover white/black/non-1 numbering, main-only inline
flow, nested/sibling variations in source order, sideline continuations independent
of role, comments/NAGs/root/description preservation, and overflow without omissions.
Renderer/hit/end-to-end tests cover every tree/reference target, inert plain prose,
selected treatment, materially reduced common-fixture pages, first/middle/last page
controls, live-board restore, unchanged progress and analysis-free behavior. Inspect
changed Gray8 images before updating checksums. Run scripts/check.sh and the documented
Kindle build. Physical e-ink readability/tap review remains a later human check.

## Implementation and validation — 2026-10-04

Worked on `phase-2`, advanced to the existing toolbar-toggle baseline `ed26376`
before implementation. `main` remains at that commit; no merge/release to main.

Notation tests failed first on labels, absent numbering and misplaced variation
order. The FEN accessor test failed because the accessor did not exist. A long-word
geometry test then exposed existing clipped prose; the flow now wraps oversized
words into measured fragments instead of losing their suffixes.

The iterative renderer work stack emits first-child SAN, NAGs and braced content,
then sibling variations in source order, then resumes the first-child continuation.
Nested branches use the same rule; role metadata never drives presentation.
Numbering uses each parent node's parsed FEN active color and retained fullmove
number. Black moves are numbered at starts/resumptions after comments or RAVs;
uninterrupted white/black pairs use conventional `1. e4 e5` notation.

Tree SAN and authored move references remain separate fragments carrying their
original node indexes. Existing outlined/inverted chip rendering and padded hit
rectangles are reused. Tests explicitly verify ambiguous padding is inert and
plain move-looking prose has no target. All three imported book fixtures fit on
one Scribe page; the nine-move main-only fixture occupies one flow line rather than
nine rows. A deliberate long explanation exercises eight-page first/middle/last
snapshots; overflow integration proves each move/reference appears exactly once,
Close remains reachable and all taps preview exact stored positions without
changing live solving state or progress bytes.

Reviewed exported Gray8 images for main-only, nested RAVs, promotion, selected
moves and first/middle/last overflow pages before replacing checksum expectations.
The existing closed-rich and analysis-free snapshots are unchanged. Optional
`ANALYSIS_SNAPSHOT_DIR` exports review PGM images from the rendering snapshot test.

`scripts/check.sh` passed formatting, strict Clippy, workspace tests, asset checks,
Python converter/format/update suites and platform/lifecycle/packaging contracts.
`scripts/build-kindle.sh` passed its full gate and the ARMv7/glibc-2.35 release
build. Host Python tooling used a temporary environment with the pinned repository
requirements. Release binary SHA-256:
`f1cc7aef1ccd588f6365722aec2040360119af1fee2aded2efb05fb107fdf419`.

No schema, converter, application-state, grading, progress, device input or toolbar
API changes. Core adds only the validated `FenPosition::fullmove_number()` accessor.
No uploaded collections were modified. No deployment or physical Scribe check was
performed; final physical e-ink readability/tap review remains a later human check.

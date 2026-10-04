# Task 33 — Borderless unselected analysis moves

**Status:** Implemented
**Depends on:** completed Task 32
**Validation:** host automated tests and reviewed Gray8 snapshots; no physical checkpoint
**Working branch:** main (post-phase-two improvement)

Render unselected tappable analysis moves and explicit prose move references as
bold text without a rectangular outline. Keep selected moves inverted inside
their existing rectangle. Preserve text weight, layout, pagination, padded hit
areas, plain-prose behavior, grading and durable progress.

Acceptance: start with a failing renderer test proving unselected tree moves
and inline references have white borders. Verify bold labels, selected inversion
and unchanged hit geometry. Review all changed analysis snapshots, run the full
host gate and Kindle cross-build, stage and deploy through the existing installer.
Verify installed binary and unchanged collection/progress hashes. Physical
readability remains user review, not inferred from installation.

## Implementation and validation — 2026-10-04

The border regression test failed first on a black unselected tree-move edge.
Removing the outline from the shared move renderer makes tree moves and explicit
prose references borderless while retaining existing bold glyphs, padding and
selected black backgrounds. Focused rendering and hit-testing suites passed.
Reviewed all nine before/after analysis snapshots, including selected moves,
variations and first/middle/last pages. Every changed pixel is a black outline
pixel becoming white in the analysis panel; glyphs and selected backgrounds are
identical. Middle/last prose-only pages remain byte-identical.

The full `scripts/build-kindle.sh` gate passed, including formatting, clippy,
workspace/snapshot tests, Python/platform/packaging contracts and the validated
ARMv7/glibc-2.35 cross-build. Staged and deployed through the documented scripts
at `root@192.168.1.20:2222`. Verified the running supervisor/app identities,
then sent TERM to that supervisor through the documented cleanup route before
installation. The app was stopped and its lock absent before/after installation;
Xorg/awesome remained present afterward. Installed binary SHA-256 matches:
`f63264af763fff428560de70b7e11739d80654a0791599e5b72728a4393a316e`.
The device loader resolves all dependencies. All five collection hashes are
unchanged; progress before/after remains
`61f5aef3786faa56ae8d2396c0447dfb5312272d765787ccff20d648456fd9c6`.
The user verified the deployed change on the Kindle on 2026-10-04 and
requested commit and push. Physical visual acceptance is confirmed.

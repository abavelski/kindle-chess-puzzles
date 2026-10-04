# Task 35 — Conventional analysis move annotations

**Status:** Implemented
**Depends on:** completed Task 34
**Working branch:** main, explicitly requested
**Validation:** host tests and reviewed Gray8 snapshots; no physical checkpoint

Render numeric PGN move-quality annotations 1–6 as `!`, `?`, `!!`, `??`, `!?`,
`?!` respectively, attached to the tree move's SAN so the move and annotation
wrap together. These cover all annotations in the four current book collections.
Preserve other numeric annotations as `$N` rather than losing their meaning.
Keep prose and explicit prose-reference labels unchanged; do not rewrite JSON,
grading, solved state or durable progress.

Start with failing tests covering the six mappings, unknown-code fallback,
multiple annotations and wrapping/tap targets/selected inversion. Review changed
analysis snapshots and a real book preview. Run scripts/build-kindle.sh (full
host gate and cross-build), commit directly to main, stage/deploy through the
existing installer, and verify binary plus unchanged collection/progress hashes.

## Implementation and validation — 2026-10-04

The four cleaned book collections contain only codes 1–6 (3,623 good moves,
400 mistakes, 737 brilliant moves, 138 blunders, 33 speculative moves and 11
questionable moves). Renderer tests failed first on plain `e4`/`Qg7#` labels
instead of their annotated forms. The renderer now appends the conventional
symbol to tree-move SAN and emits only unmapped codes as separate `$N` text.
Collection annotations remain stored as numbers. Explicit reference labels and
plain prose containing `$1` are unchanged.

Focused unit, rendering/hit and end-to-end tests passed, covering all six codes,
multiple annotations, unknown-code fallback, narrow wrapping, annotation-inclusive
tap targets, selected inversion and unchanged grading/progress. Reviewed all six
annotation snapshots plus unknown/selected states, the three changed existing
analysis snapshots and real book 135b before/after. Only analysis-panel pixels
change; unknown-code snapshots and unaffected views remain identical.

The complete scripts/build-kindle.sh gate passed: formatting, clippy, workspace
and snapshot tests, Python/platform/packaging contracts and validated
ARMv7/glibc-2.35 release build. Deployment uses the existing runtime installer;
no collection data update is needed for this renderer-only change.


## Deployment — 2026-10-04

Implementation `c762853` committed directly to main and pushed. Used the existing
scripts/stage-kindle.sh and scripts/deploy-kindle.sh at root@192.168.1.20:2222.
Initial inspection found the app running; it exited during host verification,
and subsequent preflight confirmed it stopped with the supervisor lock absent.
No process signal was sent. App/lock remained absent after installation, with
native Xorg/awesome present.

Installed binary SHA-256 matches the validated release:
`211e637917fbdcf4311605deb7da227164e0767cf61fcc114f24933077877adc`.
The device loader resolves all dependencies. All five collection hashes match
the Task 34 values; progress before/after remains
`77fff6499aae93b3baccccf59d75989d4cf9f2b70074b50daf71e9c034f538ef`.
Ready for library launch; physical visual review remains with the user.

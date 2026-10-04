# Task 29 — Compact analysis toolbar toggle

Status: Implemented (host acceptance; device interaction pending user testing)

Dependencies: completed Tasks 23–25 and 28.

Scope: move analysis from the description area into the first, compact icon-only
button in the existing toolbar. Match the other toolbar button heights, preserve
minimum touch size, show a selected monochrome state while browsing, and use the
same button to close analysis. Remove the analysis panel's Close control.
Legacy puzzles show an unavailable analysis button. Keep preview and durable
progress separate. Work directly on main as explicitly requested by the user.

Acceptance: host tests cover geometry, toggle opening/closing and restoring the
live board without progress mutation, unavailable legacy puzzles, and rendered
open/closed states. Inspect changed snapshots, run repository quality gates,
cross-build, and deploy the committed build to the Scribe for user testing.
Physical readability and finger/stylus interaction are pending user testing;
deployment is not evidence those checks passed.

Validation record (2026-10-04): geometry regression failed before implementation;
toggle regression failed because the action was absent. Both pass after implementation.
Reviewed grayscale renders of rich analysis closed/open and unavailable legacy analysis;
updated snapshot hashes for the six-button toolbar and two-control analysis footer.
The compact control has a 118×118 px touch target at 300 DPI, with a 94 px visual
height matching its neighbors. Closing preview restores complete live application
state and does not emit persistence effects. Cross-build/deployment results are
recorded in the device notes when available.

Release validation: `scripts/build-kindle.sh` passed formatting, clippy, the full
workspace suite, Python/C/lifecycle/package contracts, asset checks, and the
ARMv7/glibc-2.35 release build. Binary SHA-256:
`dae7826d30e0aa4ec0f67603d3afb1f32a20bbe20367c2a547aa78771bb08fab`.

Deployed successfully to the first-generation Scribe on 2026-10-04. Installed
binary hash matches the tested release; all five collection hashes and progress
hash are unchanged. Physical interaction remains pending user testing. See
`docs/device/ks1-barolo.md` for the deployment record.

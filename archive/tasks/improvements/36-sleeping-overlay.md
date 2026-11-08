# Task 36 — Sleeping board overlay

**Status:** In progress — implementation complete; physical overlay review pending
**Depends on:** completed Task 07 lifecycle and Task 35
**Working branch:** main, explicitly requested

Display a KOReader-style white rectangle with a black border and “Sleeping...”
centered over the existing board when powerd announces screensaver entry. Remove
it on wake without changing puzzle, analysis, grading or durable progress. Listen
passively; do not change stock services, power policy, rotation or suspend commands.
Keep renderer platform-neutral and use damage-aware presentation.

Start with failing renderer/snapshot, power-event transition and input timeout
contract tests. Run the full host gate and Kindle cross-build. Commit to main
and deploy via the existing installer, checking runtime and user-data hashes.
Physical visibility of the new overlay is a user checkpoint after deployment;
record it as pending until observed. The user confirmed existing idle/button
sleep and wake on 2026-10-04.

## Implementation and automated validation — 2026-10-04

Renderer and power-event tests failed first on missing APIs; C polling contracts
failed first on the missing power descriptor. Added platform-neutral board overlay,
passive power adapter and finger/pen/power multiplexing. Sleeping ignores taps;
wake restores the existing view through damage tracking. Reviewed the Scribe
Gray8 snapshot (`11497567903300852943`); pixels outside the overlay remain equal
and the app/progress are unchanged. Existing snapshots remain green.

The full scripts/build-kindle.sh gate passed using an isolated host environment
with requirements-tools.txt: fmt, clippy, workspace tests, renderer snapshots,
asset checks, Python converter/collection tests, C/input/lifecycle/package
contracts and ARMv7/glibc-2.35 release validation. The device listener timeout
exit 255 was verified and handled. No power policy or stock lifecycle transition
was introduced. Physical overlay appearance/removal is pending after deployment.

## Deployment — 2026-10-04

Committed implementation `4ee7d7b` directly to main. Staged and deployed with
scripts/stage-kindle.sh and scripts/deploy-kindle.sh to root@192.168.1.20:2222.
Installer succeeded. Installed runtime SHA-256 matches the validated release:
`9a6781ed6c05a220da84ea38c8ca9986c88c68b1f93e62739e86cb03ced19e94`.
The device loader resolves all dependencies. All five puzzle collection hashes
and progress hash match their pre-deploy values. App and supervisor lock are
absent; Xorg/awesome are present and powerd remains active. App is ready for
library launch. New overlay visibility and wake removal remain pending user review.

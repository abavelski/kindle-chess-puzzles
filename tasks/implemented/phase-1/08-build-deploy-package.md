# Task 08 — Make build, deployment, and Kindle packaging reproducible

**Status:** Implemented — post-reboot library observation explicitly deferred at phase-one close

**Depends on:** Tasks 04-07

## Outcome

A fresh checkout can run checks, cross-build the Scribe binary with pinned dependencies, stage a complete install, and launch it through the user's modern Kindle homebrew stack.

## Reproducible toolchain

Document and automate:

- Rust toolchain/version;
- confirmed Kindle Rust target;
- C cross-compiler/sysroot;
- pinned FBInk revision;
- required FBInk features/build flags;
- asset generation;
- final link strategy;
- licenses/notices.

Avoid relying on globally installed mystery libraries.

A containerized/devcontainer build is welcome if it makes the toolchain reproducible, but keep the inner commands understandable.

## Build scripts

Add scripts or an `xtask` for:

- `check`: fmt + clippy + tests + snapshots;
- `build-kindle`: generate assets, build FBInk, build Rust binary;
- `stage-kindle`: create an installable/stage directory without touching a device;
- `deploy-kindle`: copy/update safely using the verified transfer mechanism.

Build should fail before deployment if tests fail.

## Staged layout

Separate:

- executable/runtime;
- source puzzle directory;
- progress/state;
- logs;
- launch script/package metadata.

Deployment must not overwrite user puzzle collections or progress unless explicitly requested.

## Scriptlet/KPM

Current KindleModding guidance uses scriptlets/KPM rather than treating KUAL as the modern default.

Provide the launch integration that matches the target jailbreak from Task 00.

If KPM is available/selected, package with a manifest and `launch.sh` following current KPM conventions. If a simple scriptlet is better for the user's setup, support that first and keep KPM as the packaged form.

The package launcher should call the tested lifecycle wrapper from Task 07 rather than duplicate service logic.

## Installation/update safety

Requirements:

- idempotent reinstall/update;
- preserve user data;
- validate architecture before launching;
- log launch failures;
- do not install system-wide files unrelated to the app;
- uninstall removes app-owned runtime/package files but preserves user puzzle/progress by default or explicitly asks/document behavior.

## CI

CI should at least:

- run host checks/tests;
- verify generated assets are up to date;
- cross-build the Kindle binary/toolchain path if practical;
- build the staged/package artifact.

## HUMAN CHECKPOINT F

From a clean local checkout:

1. run the documented check;
2. build the Kindle artifact;
3. stage/deploy using only documented commands;
4. launch from the intended Kindle UI/scriptlet mechanism;
5. exit;
6. update to a second build without losing puzzles/progress;
7. repeat launch after reboot;
8. verify uninstall/reinstall data policy as documented.

## Acceptance criteria

- no manual source edits are required to build;
- FBInk revision is pinned;
- package/stage contents are inspectable and deterministic enough to review;
- deployment preserves user data;
- modern launch mechanism works on the target Scribe;
- documentation matches actual commands.

## Suggested commit

`build: add reproducible Kindle build and package workflow`

## Implementation record

Build/stage/deploy automation, Scriptlet/KPM payloads, 13 host package/build
contracts and CI artifacts are implemented. A clean local checkout passed the
documented checks, pinned ARMv7/glibc-2.35 build and stage workflow. scp deployment,
a second independent build installed through KPM 0.2.2, and actual KPM
uninstall/reinstall all passed without changing puzzle or progress hashes.

Commands and verified installation/data policy are in
[BUILD_DEPLOY.md](../docs/BUILD_DEPLOY.md); evidence is recorded in
[the device document](../docs/device/ks1-barolo.md). Task 07 is committed/pushed
with the user-authorized natural suspend/resume deferral, which packaging tests
do not claim to resolve.

The library launch/exit observation subsequently passed with the final paired-process
display handoff. The only unperformed checkpoint-F item is library launch after a full
device reboot. At phase-one close on 2026-10-03 that observation is explicitly deferred
and remains **unverified**; Task 08's Implemented status does not claim a reboot test.
No reboot result is inferred from host, loader, package-install, or ordinary relaunch
checks.


Library checkpoint regression (2026-10-03): the user reported blocked native
swipe/menu access and native white overpainting while the app still accepted
taps. Supervisor termination restored normal stock UI/touch, confirmed by the
user. Exclusive finger input requires an explicit in-app Exit control; the
previous UI had none. Add host-tested Exit routing and progress flush, then
verify normal supervisor cleanup through the library launch. Investigate native
launcher repaint timing before choosing a display ownership handoff; do not
mask the problem with full-screen redraws on every tap. Reboot and the separately
deferred natural suspend/resume check remain outstanding.


Top-right X fix deployed on 2026-10-03: host tests cover top-right geometry,
modal exit routing and progress flush; updated render/parity snapshots were
reviewed. Full checks and pinned release build passed, documented stage/deploy
commands installed the matching binary, and device hashes confirmed unchanged
puzzles/progress. The user confirmed the deployed X exit works on 2026-10-03.
Later display, reboot and suspend issues were not included in this fix.


White-overlay root cause verified on 2026-10-03: winmgr Active App T0 timeout
repaints the blank native X background while the SH_Integration child remains
alive. A timed awesome STOP/CONT probe kept the framebuffer unchanged beyond
that timeout and restored the window manager. The supervisor now owns only a
verified running awesome/Xorg PIDs and start times, pauses them before chess and resumes them
before native exit repaint. Host regressions cover startup order, normal/TERM/
child-KILL restoration, refusal of already stopped display processes and PID reuse.
Library/reboot observations remain required; natural suspend/resume stays deferred.

The initial awesome-only implementation failed a rapid-relaunch device probe:
pending Xorg drawing still covered the board. A second bounded probe paused
both awesome and Xorg and preserved the complete frame across consecutive
launches. The final supervisor validates both processes before either STOP,
then resumes Xorg before awesome during cleanup.

Final paired-process fix deployed: full host checks/release build passed and
two installed-device relaunch tests retained the complete chess frame past the
native timeout. Supervisor TERM and child KILL resumed both native processes
in order and released app ownership; puzzle/progress hashes stayed unchanged.
User library visual/tap/X-exit confirmation and reboot observation are pending.

User confirmed “all worked fine” after the final library regression test on
2026-10-03: board visibility, taps/promotion, X exit and native UI restoration
passed. Logs/framebuffer capture corroborate the launch and normal cleanup.
Only post-reboot launch remains unverified for checkpoint F. Phase-one closure records
that observation as deferred rather than blocking the completed build/deploy/package
workflow. The separately deferred natural suspend/resume test is unchanged.

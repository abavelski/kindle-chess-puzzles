# Task 08 — Make build, deployment, and Kindle packaging reproducible

**Status:** In progress — library launch/exit and reboot checkpoint pending

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

The remaining checkpoint F checks are library launch/exit and post-reboot
library launch. Task 08 must not be marked Implemented until those observations
are recorded. No reboot or successful library launch is inferred from host,
loader, or package-install checks.

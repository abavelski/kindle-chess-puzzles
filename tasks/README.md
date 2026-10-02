# Implementation tasks

These tasks are ordered to keep uncertainty isolated: first measure the Scribe, then build/test shared logic on the host, then add FBInk and input, then complete feature parity, and only afterward optimize lifecycle/refresh and packaging.

Implement one task at a time.

| Task | Outcome | Status |
| --- | --- | --- |
| [00](00-device-probe.md) | Record Scribe/FBInk/input/lifecycle facts without app code | Implemented |
| [01](01-bootstrap-and-core.md) | Rust workspace, CI/checks, board/FEN/UCI/puzzle/progress core | Implemented |
| [02](02-application-state.md) | Pure test-driven puzzle-solving application state machine | Implemented |
| [03](03-renderer-and-assets.md) | Shared deterministic e-ink renderer, layout, hit testing, Sashité assets | Implemented |
| [04](04-fbink-and-input.md) | Kindle FBInk display + touch input + minimal event loop | Implemented |
| [05](05-parity-features.md) | Promotion, Free Board, navigation, flip/lock, descriptions, feedback | Implemented |
| [06](06-persistence-and-collections.md) | Multi-collection workflow and durable progress on Kindle | Ready |
| [07](07-eink-lifecycle.md) | Damage/refresh policy plus safe Kindle launch/exit lifecycle | Ready |
| [08](08-build-deploy-package.md) | Reproducible cross-build, deploy, scriptlet/KPM package | Ready |
| [09](09-parity-validation.md) | Full reference-parity and reliability validation | Ready |

## Milestones

### M0: hardware facts

M0 is complete: HUMAN CHECKPOINT A passed on 2026-10-02. Task 00 records verified Scribe architecture/ABI, framebuffer/touch geometry, finger/pen separation, FBInk overlay behavior, and normal recovery in `docs/device/ks1-barolo.md`. Later build/lifecycle/storage validation remains scoped to its own tasks.

No product implementation should depend on guessed event nodes, resolution, ABI, or lifecycle commands.

### M1: host-complete application

M1 is complete. Tasks 01-03 provide the Rust workspace, compatibility core, fixtures, pure application state machine, deterministic Gray8 renderer, DPI-aware layout/hit testing, Sashité assets, snapshot coverage, quality gate, and ARMv7 cross-compilation smoke test. Almost all application behavior is now executable on the host without the Kindle.

### M2: first usable Scribe build

Task 04 is complete: the shared renderer now runs through pinned FBInk on the physical Scribe with capability-discovered finger input and verified coordinate mapping/recovery. Task 05 is complete: HUMAN CHECKPOINT C verified single-collection solving, promotion, navigation, Free Board, orientation, descriptions, and normal recovery on 2026-10-02. Task 06 then adds collection/progress persistence. This is the functional parity milestone.

### M3: reliable e-ink application

Tasks 07-09. Optimize refresh behavior, make launch/deploy reproducible, and prove repeated real-device use.

## Common definition of done

Every task must:

- follow `AGENTS.md`;
- start behavior with failing tests where host-testable;
- keep core/render code Kindle-independent;
- keep `cargo fmt --check`, clippy, and tests green once the Rust workspace exists;
- preserve version-1 puzzle compatibility;
- avoid chess-engine behavior unless explicitly stated;
- keep all interaction states clear without color;
- add/update renderer snapshots for new visible states;
- complete any HUMAN CHECKPOINT before marking the task Implemented.

## Not in initial parity

Do not fold these old ideas into another task:

- Unsolved-only navigation;
- version-2 branching/rich solution lines;
- chess engine/legal-move validation;
- PGN;
- Kobo runtime implementation.

They can become new tasks after Task 09.

## Future Kobo rule

The initial Kindle work should make a Kobo backend possible by keeping core/render reusable. Do not add a Kobo build, deploy script, input adapter, or device lifecycle during Tasks 00-09.

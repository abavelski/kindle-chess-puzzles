# Next tasks: split Puzzles and Game Review into separate Kindle apps

**Created:** 2026-11-08. **Priority:** product/application reuse; Kobo and other readers come later.  
**Present:** one kindle-chess executable/package with two workspace modes.  
**Goal:** two genuinely independent applications that reuse common chess and e-ink components.  
**Status:** implementation underway on `app-split`; see per-task evidence.

## How Codex should execute this plan

Read [AGENTS.md](../AGENTS.md), [architecture](ARCHITECTURE.md), [testing](TESTING.md), then the detail for **one** task, plus only its linked references. Pick the first Ready task whose dependencies are Done. Move its table and detailed status to **In progress** and record branch/owner/start date. At completion, record failing test, implementation commit/PR link, commands/results and any device checkpoint; synchronize table + detailed status in the same commit. Do not implement a later task opportunistically.

**Statuses:** Ready; Blocked (name unmet dependency/issue); In progress; Waiting for device; Done. Done requires evidenced automated acceptance; physical tasks additionally require a recorded human checkpoint. If a contract assumption changes, update this plan before editing code. Prefer small, test-first commits and owner-reviewed PRs unless expressly directed otherwise. Historical tasks under [archive/tasks](../archive/tasks/) are not this queue.

**Base gates:** cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets -- -D warnings; cargo test --locked --workspace; scripts/check.sh. For package changes also build and stage the pinned release, inspect SHA256 receipt and package. After the second real binary exists, all target checks must run for both apps. Host results do not establish Scribe/Bluetooth behavior.

## Task status index

| ID | Deliverable | Depends on | Status | Human device check |
| --- | --- | --- | --- | --- |
| SPLIT-00 | Documentation/archive and guarded CLI selector | — | Done (CI verification pending) | No |
| SPLIT-01 | Freeze golden compatibility baseline | 00 | Done | No |
| SPLIT-02 | Generic chess model and analysis validation | 01 | Done | No |
| SPLIT-03 | Puzzle-only product state/actions | 02 | Ready | No |
| SPLIT-04 | Review-only product state/actions | 02 | Ready | No |
| SPLIT-05 | Shared UI versus product layouts/hit actions | 03,04 | Blocked | No |
| SPLIT-06 | Domain-free Kindle platform/adapters | 03,04,05 | Blocked | No |
| SPLIT-07 | Shared runtime and exclusive device ownership | 05,06 | Blocked | No |
| SPLIT-08 | Two actual Cargo binaries | 07 | Blocked | No |
| SPLIT-09 | Separate state, settings and legacy migration | 08 | Blocked | No |
| SPLIT-10 | Per-app cross-build/stage/receipts | 08,09 | Blocked | No |
| SPLIT-11 | Independent install/KPM/scriptlets/deploy | 10 | Blocked | No |
| SPLIT-12 | Dual-app CI and regression matrix | 10,11 | Blocked | No |
| SPLIT-13 | Physical Scribe dual-app acceptance | 12 | Blocked | Yes |

SPLIT-03 and SPLIT-04 may be parallelized after 02 if their changes do not collide; SPLIT-07 integrates the shared runtime. All other tasks should follow dependencies rather than numeric convenience.

## Task details

### SPLIT-00 — Documentation, archive, transitional CLI

**Status:** Done (source change prepared, CI confirmation pending). **Owner:** 2026-11-08 preparation commit. **Human gate:** none.  
**Scope:** archive completed and obsolete documents plus the entire former tasks tree; simplify root AGENTS; update docs index/architecture; add app selector scripts/app.sh and tools/kindle_build.py --app flag; add selector tests to host check. Existing build-kindle/stage-kindle/deploy-kindle aliases remain stable. An unimplemented review product must **fail closed**, never install a renamed copy of the combined binary.  
**Evidence still needed:** run python3 -m unittest tests/test_app_cli.py; sh -n scripts/app.sh; scripts/check.sh in CI. Add CI link here.

### SPLIT-01 — Behavioral baseline and contracts

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [puzzle format](PUZZLE_FORMAT.md), [review behavior](GAME_REVIEW.md), [physical board](PHYSICAL_BOARD.md), [settings](SETTINGS.md).  
**Likely files:** crates/chess-core/tests, crates/chess-render/tests, app/kindle-chess/src/main.rs tests, tests/test_packaging.py, tests/fixtures.  
**First failing tests:** two independent sessions demonstrate that switching/review navigation leaves serialized puzzle progress unchanged and puzzle navigation leaves review resume unchanged; snapshot puzzle grading/promotions/analysis and review mainline/variation/FREE, settings migration, Chessnut input/seat and sleep/exit, plus stable app/package identities.  
**Acceptance:** add named golden compatibility cases, record their exact commands and outputs here; existing combined app still passes full host check. No production behavior change. **Fit:** Codex.

**Evidence (implementation: this task commit, `test(split): freeze compatibility baseline`):** Added `split_golden_two_restored_sessions_keep_progress_and_resume_independent`, exact serialized solved-progress fixture, stable installed identity test, and [named golden matrix](../tests/fixtures/split-baseline/README.md). Initial focused compile failed because the golden fixture did not exist; replacing its contents with `{}` then produced the expected byte assertion failure (0 passed / 1 failed). Restoring the frozen v1 bytes: `cargo test --locked -p chess-core --test split_baseline` → 1 passed. `python3 -m unittest tests/test_split_baseline.py` → 1 passed. `PATH=/tmp/kindle-split-venv/bin:$PATH scripts/check.sh` → exit 0 (fmt, locked clippy/workspace tests, existing reviewed checksums/damage reconstruction, Python/tooling/lifecycle/package/selector gates). Host Python dependencies installed from `requirements-tools.txt` in an external temporary venv. No production behavior or package change; no device claim.

### SPLIT-02 — Common chess model + generic analysis validation

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [architecture](ARCHITECTURE.md), [puzzle format](PUZZLE_FORMAT.md), [review format](GAME_REVIEW_FORMAT.md).  
**Likely files:** chess-core board.rs/fen.rs/uci.rs/analysis.rs/puzzle.rs/review.rs, crate Cargo.toml files.  
**First failing tests:** review-only analysis tree without a puzzle solution is valid; a puzzle with legacy solution/tree mismatch is still rejected.  
**Implementation:** separate reusable Board/FEN/UCI/analysis tree from puzzle-specific compatibility projection. Introduce a clean chess-model crate or comparable proven boundary; keep public compatibility adapters during migration.  
**Acceptance:** no product/Kindle imports in generic model, no format/JSON fixture changes, full tests green. **Fit:** Codex.

**Evidence (implementation: this task commit, `refactor(split): extract product-neutral chess model`):** Extracted `chess-model` with Board/FEN/UCI, generic tree/span validation, main-line projection and role membership. Compatibility module/type re-exports remain in chess-core. Puzzle legacy-solution comparison now lives in puzzle.rs; Review's non-empty main-line rule lives in review.rs. `cargo test --locked -p chess-core --test model_boundary` first failed with missing `chess_model`; after extraction → 2 passed (review tree without solution; generic acceptance versus puzzle rejection of mismatched solution). `cargo check --locked -p chess-model` and `cargo tree --locked -p chess-model` passed; only serde_json dependencies, no product/platform imports. Frozen formats/fixtures/checksums unchanged. Full `PATH=/tmp/kindle-split-venv/bin:$PATH scripts/check.sh` → exit 0. An earlier run's rustdoc artifact mismatch from a concurrent rebuild was resolved by rerunning the gate without concurrent source/build changes. No package/platform change or device claim.

### SPLIT-03 — Puzzle-only state machine and effects

**Status:** Ready. **Read:** [puzzle format](PUZZLE_FORMAT.md), [physical board](PHYSICAL_BOARD.md), [settings](SETTINGS.md).  
**Likely files:** chess-core app.rs, puzzle.rs, progress.rs, collection.rs, board_input.rs, board_session.rs, settings.rs; new crates/puzzle-app.  
**First failing tests:** create/dispatch puzzle state with no review file/library or review actions; exact stored-line grading, wrong-move rollback, promotions, solved IDs, FREE board, collections, physical-board intent and puzzle preferences still work.  
**Acceptance:** puzzle product cannot own review picker/resume/toggle; preserves v1 data/progress; no platform dependency. **Fit:** Codex.

### SPLIT-04 — Review-only state machine and effects

**Status:** Ready. **Read:** [review behavior](GAME_REVIEW.md), [review format](GAME_REVIEW_FORMAT.md), [review import](GAME_REVIEW_IMPORT.md).  
**Likely files:** chess-core review.rs/review_state.rs/review_resume.rs/app.rs; new crates/review-app.  
**First failing tests:** construct/dispatch review from games only; PREV/NEXT, variation preview, FREE/RESET, missing-library warnings and resume work without puzzle AppState or Progress.  
**Acceptance:** independent review actions and persistence contract; no grading/solved-ID state or platform coupling. **Fit:** Codex.

### SPLIT-05 — Shared e-ink UI, product-specific screens

**Status:** Blocked by 03,04. **Read:** [architecture](ARCHITECTURE.md), [testing](TESTING.md).  
**Likely files:** chess-render canvas.rs, geometry.rs, pieces.rs, font.rs, analysis_panel.rs, layout.rs, render.rs, button.rs, damage.rs; new generic eink-ui and product screen modules.  
**First failing tests:** generic board/analysis components render without importing any product action type; product-specific hit testing maps to only that product's actions, preserving old STANDARD/SMALL snapshots and damage reconstruction.  
**Acceptance:** no puzzle or review actions in shared UI; retain monochrome readability, touch targets, text pagination, explicit move-ref interactions. **Fit:** Codex.

### SPLIT-06 — Untangle platform from products

**Status:** Blocked by 03,04,05. **Read:** [Kindle notes](KINDLE_SCRIBE.md), [lifecycle](EINK_LIFECYCLE.md).  
**Likely files:** kindle-platform storage.rs, refresh.rs, chessnut.rs, input.rs, display.rs, power.rs; product repository adapters.  
**First failing tests:** Kindle platform compiles with no chess-core/puzzle-app/review-app dependency; generic file operations and refresh requests have no puzzle/review-aware parameters.  
**Implementation:** move format discovery/loading, progress/resume handling and chess-aware refresh/board intent to product/composition; preserve low-level hardware and Bluetooth transport.  
**Acceptance:** no imports of Puzzle, ReviewGame, Progress or AppState in platform; data writes remain conservative. **Fit:** Codex.

### SPLIT-07 — Runtime composition and exclusive display ownership

**Status:** Blocked by 05,06. **Read:** [lifecycle](EINK_LIFECYCLE.md), [physical board](PHYSICAL_BOARD.md).  
**Likely files:** app/kindle-chess/src/main.rs, scripts/kindle_launch.sh, common runtime helper, tests/test_lifecycle.py.  
**First failing tests:** two product processes cannot simultaneously own framebuffer/paused Xorg; second launch refuses safely without disturbing first, and abnormal exits restore device ownership.  
**Implementation:** reuse event/input/power/present/shutdown wiring; add one global display lock plus namespaced app locks. Keep Puzzles-only Chessnut process lifecycle.  
**Acceptance:** scoped cleanup on all exits; no duplicated supervisor or two display writers; documented SSH recovery. **Fit:** Codex/high risk.

### SPLIT-08 — Real Cargo app entrypoints

**Status:** Blocked by 07. **Read:** [architecture](ARCHITECTURE.md).  
**Likely files:** Cargo.toml/Cargo.lock, new apps/kindle-puzzles and apps/kindle-game-review, old app/kindle-chess.  
**First failing tests:** cargo build --locked -p kindle-puzzles and cargo build --locked -p kindle-game-review both work, and independent startup wiring never loads the other's library/state.  
**Acceptance:** two distinct product binaries, one domain each; no hidden workspace switch, no puzzle data initialization in Review or game picker in Puzzles. Retire legacy entrypoint only when compatibility gates are green. **Fit:** Codex.

### SPLIT-09 — Isolated data paths and safe migration

**Status:** Blocked by 08. **Read:** [settings](SETTINGS.md), [review import](GAME_REVIEW_IMPORT.md).  
**Likely files:** runtime/storage adapters, product settings/resume, migration tests, docs/BUILD_DEPLOY.md.  
**First failing tests:** fresh install; legacy-only games and review resume; both old/new present; malformed/future schema; interrupted migration; rollback; stable game IDs; puzzle progress bytes identical.  
**Decision required inside task:** choose and document a **tested one-time copy** or **read-only legacy fallback**, not silent relocation. Retain Puzzles root /mnt/us/kindle-chess and existing puzzles/progress/settings. Proposed separate Review root /mnt/us/kindle-game-review with games/, state/review-resume.json and its own settings.  
**Acceptance:** preserves old games at /mnt/us/kindle-chess/games, old review resume/settings and puzzle data without rewriting/deleting; validates ID paths and backup/recovery. **Fit:** Codex.

### SPLIT-10 — Product-aware pinned build and staging

**Status:** Blocked by 08,09. **Read:** [build/deploy](BUILD_DEPLOY.md).  
**Likely files:** tools/kindle_build.py, tools/chessnut_build.py, scripts/app.sh and wrapper scripts, tests/test_packaging.py.  
**First failing tests:** app.sh build/stage puzzles and app.sh build/stage review produce different validated ELF binaries, unique receipt app IDs, artifacts and stages; a cross-app binary/receipt substitution is rejected.  
**Implementation:** replace the current intentional Review guard with an explicit product registry (Cargo package, binary, source digest, receipt, stage, manifest, runtime root), preserving pinned Zig/FBInk/glibc 2.35 and source licensing. Review must not bundle Chessnut worker if unused.  
**Acceptance:** verified independent reproducible stages, complete checksums/source archives; legacy puzzle scripts remain compatible. **Fit:** Codex/high risk.

### SPLIT-11 — Independent KPM, scriptlets, install and deploy

**Status:** Blocked by 10. **Read:** [build/deploy](BUILD_DEPLOY.md), [lifecycle](EINK_LIFECYCLE.md).  
**Likely files:** packaging/kindle manifests, install/uninstall/launch/scriptlets, tools/kindle_build.py deployment, tests/test_packaging.py.  
**First failing tests:** install A and B, update/uninstall A, assert B's binary/scriptlet/runtime/data all byte-identical; repeat reversed; verify corrupt archive rejection and safe recovery.  
**Compatibility:** preserve puzzle KPM ID kindle_chess, scriptlet kindle-chess.sh, /mnt/us/kindle-chess. Give Review distinct KPM ID kindle_game_review, scriptlet kindle-game-review.sh, /mnt/us/kindle-game-review, own lock/log paths. Global framebuffer lock remains shared.  
**Acceptance:** no cross-product deletes or overwrites, no user-data deletion, matching manifest and deployed source attestation. **Fit:** Codex/high risk.

### SPLIT-12 — Host/CI two-product matrix

**Status:** Blocked by 10,11. **Read:** [testing](TESTING.md), [build/deploy](BUILD_DEPLOY.md).  
**Likely files:** .github/workflows/ci.yml, scripts/check.sh, product tests and packaging tests.  
**First failing tests:** CI fails if one artifact/receipt/manifest missing, wrong app selected, wrong binary staged or Review unexpectedly bundles puzzle/Chessnut assets.  
**Acceptance:** single host regression suite and both product ARMv7/glibc-2.35 build/stage jobs pass from clean checkout; publish distinct package names/receipts; no physical claim. **Fit:** Codex.

### SPLIT-13 — Human Scribe checkpoint for both apps

**Status:** Blocked by 12. **Human required:** yes. **Read:** [device record](device/ks1-barolo.md), [lifecycle](EINK_LIFECYCLE.md), [physical board](PHYSICAL_BOARD.md).  
**Change:** append dated acceptance matrix and artifact hashes to docs/device/ks1-barolo.md; update BUILD_DEPLOY.md and mark tracker Done only when signed off.  
**Device checklist:** firmware/commit/checksum; install both, independently library-launch/exit; Puzzles grading/Free Board/analysis/persisted progress/settings and opt-in Chessnut Send/seat/input; Review imported games/PREV/NEXT/variations/FREE/resume/migration; pause/sleep/repeated wake/native repaint; global double-launch refusal; update/uninstall each while preserving the other's runtime and data; roll back using backup and SSH recovery. Clearly mark automatic Chessnut reply motion/STOP and any other unverified item as pending, never assumed.  
**Acceptance:** real hardware observations recorded with commands, results, hashes and user approval. **Fit:** Codex + owner.

## Completion definition and deferred work

Complete only after two independent apps build/deploy from the same repo, share chess/e-ink/Kindle components, preserve existing data and physical behavior, cannot concurrently own the screen, and pass both automated and human gates. **Not in scope:** Kobo platform, more chess products, or feature expansion unrelated to splitting.

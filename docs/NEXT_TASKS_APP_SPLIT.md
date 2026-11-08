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
| SPLIT-03 | Puzzle-only product state/actions | 02 | Done | No |
| SPLIT-04 | Review-only product state/actions | 02 | Done | No |
| SPLIT-05 | Shared UI versus product layouts/hit actions | 03,04 | Done | No |
| SPLIT-06 | Domain-free Kindle platform/adapters | 03,04,05 | Done | No |
| SPLIT-07 | Shared runtime and exclusive device ownership | 05,06 | Done | No |
| SPLIT-08 | Two actual Cargo binaries | 07 | Done | No |
| SPLIT-09 | Separate state, settings and legacy migration | 08 | Done | No |
| SPLIT-10 | Per-app cross-build/stage/receipts | 08,09 | Done | No |
| SPLIT-11 | Independent install/KPM/scriptlets/deploy | 10 | Done | No |
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

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Boundary decision:** establish typed product modules in chess-core first; extract Cargo crates once shared UI/preferences interfaces settle, preserving compatibility composition during the transition. **Read:** [puzzle format](PUZZLE_FORMAT.md), [physical board](PHYSICAL_BOARD.md), [settings](SETTINGS.md).  
**Likely files:** chess-core app.rs, puzzle.rs, progress.rs, collection.rs, board_input.rs, board_session.rs, settings.rs; new crates/puzzle-app.  
**First failing tests:** create/dispatch puzzle state with no review file/library or review actions; exact stored-line grading, wrong-move rollback, promotions, solved IDs, FREE board, collections, physical-board intent and puzzle preferences still work.  
**Acceptance:** puzzle product cannot own review picker/resume/toggle; preserves v1 data/progress; no platform dependency. **Fit:** Codex.

**Evidence (implementation: this task commit, `refactor(split): isolate puzzle product state and actions`):** `puzzle_app::PuzzleState/PuzzleAction/PuzzleEffect` owns grading, collections/progress, analysis preview, FREE, preferences and physical-board input/intent without review fields/actions. Existing combined AppState delegates Puzzles dispatch to this implementation; it retains workspace composition and the legacy multi-profile settings document. Shared promotion values moved to interaction.rs. First `cargo test --locked -p chess-core --test puzzle_product` failed on missing puzzle_app; after extraction → 2 passed, covering construction/grading and promotion rollback, FREE/analysis, preferences/seat, opt-in connect/Send/input pause/exit. Compile-fail doctest rejects `PuzzleAction::ToggleWorkspace`. All existing core tests pass, including authored replies, solved IDs, collection navigation and physical seating. `cargo clippy --locked --workspace --all-targets -- -D warnings` and full `PATH=/tmp/kindle-split-venv/bin:$PATH scripts/check.sh` → exit 0, unchanged reviewed grayscale/damage/persistence baselines. No device claim.

### SPLIT-04 — Review-only state machine and effects

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [review behavior](GAME_REVIEW.md), [review format](GAME_REVIEW_FORMAT.md), [review import](GAME_REVIEW_IMPORT.md).  
**Likely files:** chess-core review.rs/review_state.rs/review_resume.rs/app.rs; new crates/review-app.  
**First failing tests:** construct/dispatch review from games only; PREV/NEXT, variation preview, FREE/RESET, missing-library warnings and resume work without puzzle AppState or Progress.  
**Acceptance:** independent review actions and persistence contract; no grading/solved-ID state or platform coupling. **Fit:** Codex.

**Evidence (implementation: this task commit, `refactor(split): isolate games-only review product`):** `review_app::ReviewApp/ReviewAction/ReviewEffect` constructs from games/preferences only and owns library/picker errors, navigation, authored selection, scratch board, preferences and resume. It has no puzzle/progress/grading state or platform imports. Combined AppState now only composes/delegates both products and retains the legacy settings envelope; both product state representations are private. Initial `cargo test --locked -p chess-core --test review_product` failed on missing review_app; after implementation → 2 passed (games-only main-line/variation resume round-trip and transient FREE/reset/empty-library warning). Full core suite and compile-fail doctest rejecting `ReviewEffect::ProgressChanged` pass. `PATH=/tmp/kindle-split-venv/bin:$PATH scripts/check.sh` → exit 0, including unchanged STANDARD/SMALL snapshots, all import/damage/persistence and runtime tests. No schema or device claim.

### SPLIT-05 — Shared e-ink UI, product-specific screens

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [architecture](ARCHITECTURE.md), [testing](TESTING.md).  
**Likely files:** chess-render canvas.rs, geometry.rs, pieces.rs, font.rs, analysis_panel.rs, layout.rs, render.rs, button.rs, damage.rs; new generic eink-ui and product screen modules.  
**First failing tests:** generic board/analysis components render without importing any product action type; product-specific hit testing maps to only that product's actions, preserving old STANDARD/SMALL snapshots and damage reconstruction.  
**Acceptance:** no puzzle or review actions in shared UI; retain monochrome readability, touch targets, text pagination, explicit move-ref interactions. **Fit:** Codex.

**Evidence (implementation: this task commit, `refactor(split): extract shared eink widgets and product screens`):** New eink-ui crate owns Gray8/geometry, embedded fonts/pieces, neutral board/analysis widgets, neutral analysis hit targets and damage calculations. Its dependency tree is chess-model + fontdue only (no product/platform imports). Product layouts/typed hit mappings remain in chess-render; read-only ScreenState implementations render PuzzleState or ReviewApp directly. Compatibility re-exports preserve existing rendering consumers and golden checksums. First `cargo test --locked -p chess-render --test product_screens` failed on missing render_puzzles/render_review; after implementation → 2 passed (independent states; STANDARD/SMALL byte-identical legacy frames; typed hit paths reject cross-product actions and workspace switching; empty Review renders safely). `cargo test --locked -p eink-ui` → 14 passed before the final output-type compatibility adjustment; final full gate also passes its tests. `cargo clippy --locked --workspace --all-targets -- -D warnings`, `python3 scripts/generate_sashite.py --check`, and full `PATH=/tmp/kindle-split-venv/bin:$PATH scripts/check.sh` → exit 0. All reviewed snapshots and damage reconstruction unchanged. No device claim.

### SPLIT-06 — Untangle platform from products

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [Kindle notes](KINDLE_SCRIBE.md), [lifecycle](EINK_LIFECYCLE.md).  
**Likely files:** kindle-platform storage.rs, refresh.rs, chessnut.rs, input.rs, display.rs, power.rs; product repository adapters.  
**First failing tests:** Kindle platform compiles with no chess-core/puzzle-app/review-app dependency; generic file operations and refresh requests have no puzzle/review-aware parameters.  
**Implementation:** move format discovery/loading, progress/resume handling and chess-aware refresh/board intent to product/composition; preserve low-level hardware and Bluetooth transport.  
**Acceptance:** no imports of Puzzle, ReviewGame, Progress or AppState in platform; data writes remain conservative. **Fit:** Codex.

**Evidence (implementation: this task commit, `refactor(split): remove product dependencies from Kindle platform`):** kindle-platform now depends only on eink-ui/fbink-sys; display/input/power, generic byte replacement and generic refresh requests contain no product imports. New kindle-app composition adapter owns product discovery/repositories, protected progress/settings/resume stores, chess-aware clean-region decisions and Puzzles board-session orchestration over the unchanged worker transport. Existing product-aware storage/input/refresh/Chessnut tests moved with their adapter; generic device tests remain in kindle-platform. First `python3 -m unittest tests/test_platform_boundary.py` failed on chess-core/chess-render dependencies; after extraction → 1 passed. `cargo check --workspace` passed. `PATH=/tmp/kindle-split-venv/bin:$PATH scripts/build-kindle.sh` → exit 0, including full scripts/check.sh, pinned worker source/native contracts and release cross-build. `scripts/stage-kindle.sh` → exit 0, validated stage/package/receipt. Release: 3,588,728 bytes; ARMv7 hard-float, glibc 2.35, Rust 1.85.1, Zig 0.13.0, FBInk 92e1270; binary SHA256 `952e2f797a22adb6cff02387bfb3731aadbf7677382b8c488f4faf2c3fb03ba4`, source SHA256 `1d3cd6b1bbf00e813893cda309e1ad3cf15f8bd148a7f450ec7074bccfa93bfc`. No device claim.

### SPLIT-07 — Runtime composition and exclusive display ownership

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [lifecycle](EINK_LIFECYCLE.md), [physical board](PHYSICAL_BOARD.md).  
**Likely files:** app/kindle-chess/src/main.rs, scripts/kindle_launch.sh, common runtime helper, tests/test_lifecycle.py.  
**First failing tests:** two product processes cannot simultaneously own framebuffer/paused Xorg; second launch refuses safely without disturbing first, and abnormal exits restore device ownership.  
**Implementation:** reuse event/input/power/present/shutdown wiring; add one global display lock plus namespaced app locks. Keep Puzzles-only Chessnut process lifecycle.  
**Acceptance:** scoped cleanup on all exits; no duplicated supervisor or two display writers; documented SSH recovery. **Fit:** Codex/high risk.

**Evidence (implementation: this task commit, `refactor(split): share runtime and lock global display ownership`):** Extracted shared device event/input/power/damage/presentation/shutdown loop behind kindle-app::runtime::Application callbacks. Combined runtime is now a product adapter; Puzzles board service stays outside the generic loop. Supervisor acquires a global display lock before any native handoff, preserves app locks, rejects all known unsupervised product binaries and uses selected-process wake identity. Runtime requires the recorded supervisor and claims a nested framebuffer lock; scoped Drop/supervisor cleanup releases ownership. First `python3 -m unittest tests.test_lifecycle.LauncherTests.test_distinct_apps_share_display_owner_and_second_cannot_resume_first` failed because the second app exited 0; after implementation, lifecycle suite → 13 passed, including refusal without STOP/CONT/repaint, abnormal child cleanup and PID-reuse checks. Runtime ownership unit test → 1 passed; existing binary runtime tests → 6 passed. Input-grab ordering source test moved to shared runtime (same assertion); focused 2 passed. Final `scripts/build-kindle.sh` (with external host venv on PATH) and `scripts/stage-kindle.sh` → exit 0, full host gate + pinned worker/native/release/stage checks. Release 3,593,824 bytes; binary SHA256 `37da96d2be42400137ae557db14a7594d4ecb5182d27d4540458c59a83170ccd`, source SHA256 `7e17a105931c5c9f6cc2cd14c58ff09fe000f6cbd2ba4d34fb795f6f1c9646c2`. [SSH recovery](EINK_LIFECYCLE.md#split-runtime-and-global-ownership--split-07-2026-11-08) covers stale locks/supervisor SIGKILL; no physical acceptance claim.

### SPLIT-08 — Real Cargo app entrypoints

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [architecture](ARCHITECTURE.md).  
**Likely files:** Cargo.toml/Cargo.lock, new apps/kindle-puzzles and apps/kindle-game-review, old app/kindle-chess.  
**First failing tests:** cargo build --locked -p kindle-puzzles and cargo build --locked -p kindle-game-review both work, and independent startup wiring never loads the other's library/state.  
**Acceptance:** two distinct product binaries, one domain each; no hidden workspace switch, no puzzle data initialization in Review or game picker in Puzzles. Retire legacy entrypoint only when compatibility gates are green. **Fit:** Codex.

**Evidence (implementation: this task commit, `feat(split): add independent Kindle product entrypoints`):** Added thin apps/kindle-puzzles and apps/kindle-game-review Cargo binaries, separate PuzzleRuntime/ReviewRuntime composition roots and a shared initial puzzle-selection helper. Puzzles opens only puzzle collections/progress/preferences/optional board service; Review opens only games/resume/preferences under /mnt/us/kindle-game-review and never constructs a puzzle state/progress store/board service. Standalone screens remove the workspace icon/target; all other STANDARD/SMALL pixels match the baseline (explicit switch-region mask in compatibility comparison); legacy snapshot hashes remain unchanged. Initial `cargo build --locked -p kindle-puzzles` failed on missing package. Both `cargo build --locked -p kindle-puzzles` and `cargo build --locked -p kindle-game-review` now pass. `cargo test --locked -p kindle-app --test products` → 1 passed (cross-library/state ignored, independent writes, Review creates no puzzle directory); product screen tests → 2 passed. Full scripts/check.sh via pinned build → exit 0; compatibility build/stage also pass after including apps/ in corresponding source. Both independent targets cross-built with pinned Zig/FBInk settings and passed validate_elf: Puzzles 3,473,136 bytes / SHA256 `df2032d62dc389e19678df48b85f76da0a571ffc136a8c3895b8a19a1c6e541a`; Review 3,175,348 bytes / SHA256 `dc0c1b495d1aed4c56ea062a4fca65c6661913de653219d1c79c099a9e45cf25`. The compatibility entrypoint/package stays available until SPLIT-10/11; app.sh review still fails closed until per-app packaging exists. No device claim.

### SPLIT-09 — Isolated data paths and safe migration

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Decision:** read-only legacy fallback for games and absent Review settings/resume; all future writes target only the new Review root. Existing new files always take precedence, including protected malformed/future bytes. **Read:** [settings](SETTINGS.md), [review import](GAME_REVIEW_IMPORT.md).  
**Likely files:** runtime/storage adapters, product settings/resume, migration tests, docs/BUILD_DEPLOY.md.  
**First failing tests:** fresh install; legacy-only games and review resume; both old/new present; malformed/future schema; interrupted migration; rollback; stable game IDs; puzzle progress bytes identical.  
**Decision required inside task:** choose and document a **tested one-time copy** or **read-only legacy fallback**, not silent relocation. Retain Puzzles root /mnt/us/kindle-chess and existing puzzles/progress/settings. Proposed separate Review root /mnt/us/kindle-game-review with games/, state/review-resume.json and its own settings.  
**Acceptance:** preserves old games at /mnt/us/kindle-chess/games, old review resume/settings and puzzle data without rewriting/deleting; validates ID paths and backup/recovery. **Fit:** Codex.

**Evidence (implementation: this task commit, `feat(split): preserve legacy review data through read-only fallback`):** Selected/documented read-only fallback, no copy/move/marker. Independent root and settings/resume/game-directory overrides use KINDLE_GAME_REVIEW_*; default legacy root is /mnt/us/kindle-chess. Absent new inputs may read legacy sources; any matching new game file or existing new state file (including corrupt/future) takes precedence. All future state writes go only to the new root. First `cargo test --locked -p kindle-app --test review_migration` failed on missing open_with_legacy; final → 6 passed. [Migration fixtures](../tests/fixtures/review-migration/README.md) cover fresh, legacy-only, both-present, malformed/future source and destination, interrupted state, rollback and path traversal. Every legacy game/settings/resume/progress byte remains identical; own malformed/future bytes survive attempted saves; IDs/resume/review settings restore. Full scripts/check.sh via pinned scripts/build-kindle.sh → exit 0; both standalone targets independently cross-built and validated; compatibility stage → exit 0. Puzzles 3,475,872 bytes / SHA256 `9e3b7ea4037d7ba5c9109f5272c1e10138dc500cc9bd48857554e449f117c039`; Review 3,183,924 bytes / SHA256 `0e0c92f51b36f73233adc63f889d8ce324c730faf518a08085436a1b4d633d15`. [Paths, precedence and rollback](BUILD_DEPLOY.md#standalone-review-data-and-legacy-fallback--split-09). No device claim.

### SPLIT-10 — Product-aware pinned build and staging

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [build/deploy](BUILD_DEPLOY.md).  
**Likely files:** tools/kindle_build.py, tools/chessnut_build.py, scripts/app.sh and wrapper scripts, tests/test_packaging.py.  
**First failing tests:** app.sh build/stage puzzles and app.sh build/stage review produce different validated ELF binaries, unique receipt app IDs, artifacts and stages; a cross-app binary/receipt substitution is rejected.  
**Implementation:** replace the current intentional Review guard with an explicit product registry (Cargo package, binary, source digest, receipt, stage, manifest, runtime root), preserving pinned Zig/FBInk/glibc 2.35 and source licensing. Review must not bundle Chessnut worker if unused.  
**Acceptance:** verified independent reproducible stages, complete checksums/source archives; legacy puzzle scripts remain compatible. **Fit:** Codex/high risk.

**Evidence (implementation: this task commit, `feat(split): build and stage product-bound Kindle artifacts`):** Explicit registry binds Cargo entrypoint, installed binary, receipt app/package/root, manifest/scriptlet and per-app output directory. Puzzles builds kindle-puzzles and preserves installed kindle-chess/KPM kindle_chess identity; Review builds kindle-game-review/KPM kindle_game_review. App-specific source digests and embedded ELF identities reject cross-app/old-combined substitutions; Review skips worker build/stage. First new packaging test failed because source_digest had no app selector; final focused selector/Review-stage checks → 5 passed, then full packaging and host gates passed. `scripts/app.sh build puzzles`, `stage puzzles`, `build review`, `stage review` (external host venv PATH) all exit 0 with pinned ARMv7/glibc-2.35/FBInk/Zig validation. Repeated real staging/archiving of both apps produced identical SHA256 tar/KPM bytes. Puzzles binary SHA256 `b27b245dfc88e80034a9693c433af73038986fd7c0abdf727063017bc2f35528`; Review `6c9bd1d57e4d0f2b5088d2705cd13683e601fd25ac594c91c0c2421ed2bf383a`. Corresponding source/dependencies and checksums included; no Review runtime worker/libs. Review install/uninstall/launch/deploy intentionally fail closed until SPLIT-11 validates that lifecycle; build/stage are real. No device claim.

### SPLIT-11 — Independent KPM, scriptlets, install and deploy

**Status:** Done. **Owner:** Codex. **Branch:** `app-split`. **Started/completed:** 2026-11-08. **Read:** [build/deploy](BUILD_DEPLOY.md), [lifecycle](EINK_LIFECYCLE.md).  
**Likely files:** packaging/kindle manifests, install/uninstall/launch/scriptlets, tools/kindle_build.py deployment, tests/test_packaging.py.  
**First failing tests:** install A and B, update/uninstall A, assert B's binary/scriptlet/runtime/data all byte-identical; repeat reversed; verify corrupt archive rejection and safe recovery.  
**Compatibility:** preserve puzzle KPM ID kindle_chess, scriptlet kindle-chess.sh, /mnt/us/kindle-chess. Give Review distinct KPM ID kindle_game_review, scriptlet kindle-game-review.sh, /mnt/us/kindle-game-review, own lock/log paths. Global framebuffer lock remains shared.  
**Acceptance:** no cross-product deletes or overwrites, no user-data deletion, matching manifest and deployed source attestation. **Fit:** Codex/high risk.

**Evidence (implementation: this task commit, `feat(split): isolate install scriptlets and deployment lifecycles`):** Shared installer/uninstaller templates now render independent product identities and refuse another product's root; retained root markers prevent reassignment after uninstall. Puzzles legacy unmarked runtimes can upgrade. Payload validation precedes replacement, temporary runtime/scriptlet commit retains rollback, unknown runtime directories and foreign library documents remain untouched. Review launch uses its own root/log/app lock and shared supervisor/global display ownership. Deploy selects product-specific incoming/root paths and verifies installed binary/receipt/source SHA over SSH. First co-installation test failed on guarded Review installer; after implementation, both update/uninstall directions preserve the other's runtime/data/scriptlet bytes, including misrouted-root refusals. Failure-injected final scriptlet swap restores the prior runtime; legacy upgrade, unknown-directory and corrupt archive/payload cases pass. Network tests are stubbed (no device deploy). Pre-marker stage replacement initially refused the old local stage; added a failing-then-passing regression and verified-only upgrade path while publish/deploy checks remain strict. Final `scripts/app.sh build/stage puzzles` and `build/stage review` (host venv PATH) → exit 0, including full host gates and pinned builds. Strict verify_stage passes both. Puzzles KPM SHA256 `4a27768984c9b7c000893f1c035c6d9acb633179b300aa1fc9da4dc7e2e4181f`; Review KPM `0a0af52be7b41f19f4d2f2b0a2fbf3ad7d8687f23ba396413bf6b36499a826b7`. [Install/recovery contract](BUILD_DEPLOY.md#independent-installation-and-recovery--split-11). Physical acceptance remains SPLIT-13.

### SPLIT-12 — Host/CI two-product matrix

**Status:** Ready. **Read:** [testing](TESTING.md), [build/deploy](BUILD_DEPLOY.md).  
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

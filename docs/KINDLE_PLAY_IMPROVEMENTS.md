# Kindle Chess Play — post-MVP improvements (Kindle)

**Implementation in progress — common feature branch; owner tests the complete set.** Created 2026-11-09. Repository: abavelski/kindle-chess-puzzles, integration branch **main**. Baseline: [KINDLE_PLAY_MVP.md](KINDLE_PLAY_MVP.md). Companion desktop plan: [IMPROVEMENTS_PLAN.md](https://github.com/abavelski/chess-games-db/blob/master/docs/tasks/kindle-play-server/IMPROVEMENTS_PLAN.md).

## Goal and existing architecture

Make the standalone Kindle Chess Play UI match the Puzzle/Review close-button icon; let the player choose **White or Black** before starting a game; replace the “last move” text with a paginated, clickable move-history panel styled consistently with Puzzles/Game Review. Tapping a historical move displays the board *after that ply* without changing server game state; **Latest** jumps to the current authoritative FEN so the player can resume playing.

Retain a lightweight Kindle client: desktop alone validates chess moves, computes SAN, owns the full session and persists My Studies. Do not introduce a Kindle chess engine, PGN parser, local game files, or shared puzzle/review state. Keep its independent package, config/lock/runtime and the existing pinned build + e-ink refresh lifecycle; avoid altering Puzzles/Review goldens or app data.

Code anchors verified on main:
- crates/chess-render/src/play.rs: PlayLayout, PlayTarget, render_play. Today Close is text “CLOSE”, the board is always drawn White-up (false orientation), and base.status contains message + only last UCI.
- crates/chess-render/src/render.rs: Puzzles/Review share draw_close_button with an X icon; existing analysis move panel and layout styles can be reused where appropriate, without coupling Play to puzzle/review models.
- crates/chess-core/src/play.rs: Snapshot validation currently demands playerColor == white, even ply while playing, selection is White-only, and New sends only rating. Snapshot already carries ordered standard-UCI moves.
- crates/kindle-app/src/products/play.rs: PlayRuntime dispatches PlayTarget and uses an asynchronous HTTP worker; errors/reconnection replace snapshot with GET state, never optimistically commit moves.
- Existing checks: cargo test -p chess-core --test play; cargo test -p chess-render --test play; cargo test -p kindle-app --test play_http --test play_runtime; scripts/check.sh; scripts/app.sh build/stage play and the artifact verifier.

## Reuse-first implementation rule (mandatory for every FKP task)

**Prefer existing shared components and focused extensions over new parallel UI/state code.** Inspect existing Puzzles/Review implementations *and* Play before coding. Reuse generic `chess-render`, `eink-ui`, `chess-model`, `chess-core` and `kindle-app` primitives; when needed, extract a small product-neutral helper rather than copy/pasting. Preserve product ownership: do not fake a Review/Puzzles analysis tree or make Play depend on their settings, persisted state or gameplay logic. New Play-specific orchestration/targets are appropriate only when the existing generic API cannot represent the interaction. For each task, record **reused, extracted and genuinely new components**, with a short reason for new code.

| Concern | Reuse or extend | Avoid duplicating |
| --- | --- | --- |
| Close X | Existing `draw_close_button` / close-icon chrome in `crates/chess-render/src/render.rs` (extract a `pub(crate)` shared helper if necessary), and existing `Layout.exit` | Hand-drawn second X, a new close button style or new global close geometry |
| Board orientation and taps | `eink_ui::board::draw_board`, `Layout` board geometry/coordinate helpers, existing flipped-board conventions from Review and Play's current `PlayTarget::Square` | A custom Black board renderer, independent board geometry or mismatched tap transforms; extract one orientation mapping helper if none exists |
| Move list/pagination | Shared font, button, `Rect`/layout and hit-test primitives; inspect `crates/chess-render/src/analysis_panel.rs` and `eink_ui::analysis_panel` for reusable **presentation/paging** parts | A second style system, bespoke font/text clipping or fake `AnalysisView` / Review move tree. If its domain-specific model prevents direct reuse, extract only a neutral chip/list/pager primitive |
| History positions | Existing UCI-to-board replay logic already in `chess-core/src/play.rs` `Snapshot::parse`, plus `chess-model` piece/FEN/coordinate helpers | A second castling/en-passant/promotion replay algorithm, Kindle legality checking, PGN parsing or local engine; refactor replay to produce per-ply positions for both validation and browsing |
| HTTP and app lifecycle | Existing `PlayApp`, `PlayRuntime`, `PlayClient` worker, `Request`/`Snapshot`/GET error recovery and shared app runtime | A new network client/threading stack, optimistic move state or shared Review/Puzzles persistent files |
| Verification | Existing core/play, render/play, HTTP/runtime tests, e-ink snapshots and `scripts/check.sh` | Duplicate harnesses or unrelated golden-frame updates |

A visual match with Puzzles/Review matters, but **do not reuse product-specific state** just to reuse a widget. Small reusable changes to neutral rendering/layout code are preferred where they keep all existing products unchanged.

## Required desktop wire behavior

Follow the [desktop extension contract](https://github.com/abavelski/chess-games-db/blob/master/docs/tasks/kindle-play-server/IMPROVEMENTS_PLAN.md#cross-repository-api-agreement-implement-in-bkp-03-before-fkp-02fkp-03) once **BKP-03** is accepted. API version and routes remain v1. POST /api/v1/games accepts rating with optional playerColor: white/black (omitting means legacy White). Snapshot playerColor indicates the human side; fen is server-authoritative, moves are ordered UCI from initial setup, ply == moves.length, and sanMoves is a parallel SAN array when supported. Black starts with a legal Maia White move already made (ply=1, Black to move), so **odd ply can be playing**. Optional server metadata: gameId, startedAt and lastMoveAt. Never reject an otherwise valid older White-only snapshot merely for missing additive metadata/SAN; render UCI history as fallback. New user-selected Black requires the upgraded server (show an actionable error otherwise).

No optimistic commit: 200/201 and GET update the authoritative board/history together; timeout => GET before retry; 409 carries state; 422 preserves state. The UI must not locally issue Maia moves, retroactively submit past-line moves, or use a history selection as expectedPly.

Owner override (2026-11-09): implement all Kindle tasks on `feature/kindle-play-improvements`, commit directly to that common branch, and defer human verification until the complete build is ready. Per-task human/dependency holds below are superseded for implementation; desktop acceptance and device results are never inferred.

## Codex task execution

**Token-efficient CLI option:** use [`bash scripts/codex-play.sh`](CODEX_PLAY_CLI.md) from this repository. It invokes Codex on one Ready task with Sol 6.1/medium by default, then uses existing focused tests and pinned build/stage/artifact scripts outside the model. Use `verify` for a no-model check and `fix` to resume with only the relevant failing-log excerpt. Human/device verification is still required and is never automated. Do not duplicate the wrapper's full build/tests inside the Codex turn.


One task per branch/PR. Start the first **Ready** task with dependencies Done. Follow the reuse-first rule above, and document reused/extracted/new components before handoff. Before coding, read AGENTS.md, README.md, docs/ARCHITECTURE.md, docs/TESTING.md, this plan and linked desktop contract. Change the row and detailed status to In progress (owner/branch/date). Add focused failing tests/snapshots first, implement that task only, then run focused tests and scripts/check.sh; build/stage/verify Play if package/runtime changes. Preserve old Puzzles and Review golden frames. Record commands + results and commit SHA in task details. Set **Waiting for human** after automated checks; **human verification is the last step** and is never inferred from passing tests. Only owner-confirmed device observations can mark Done; update both status locations together in one commit. Keep dependent tasks Blocked until accepted.

| Order | Task | Depends on | Initial status | Deliverable |
| --- | --- | --- | --- | --- |
| 1 | FKP-01 — match existing close icon | MVP | **Implemented; final testing deferred** | Shared X-icon rendering, same close position/hit target |
| 2 | FKP-02 — choose human side | FKP-01 + desktop BKP-03 Done | Implemented; final verification pending | Side selector, rotated board/taps, server contract |
| 3 | FKP-03 — clickable move record and Latest | FKP-02 + desktop BKP-03 Done | Implemented; final verification pending | Paged SAN move panel, historical FEN viewer, return-to-live |
| 4 | FKP-04 — combined physical-device acceptance | FKP-03 + desktop BKP-04 Done | In progress (host verification) | End-to-end human verification/evidence only |

### FKP-01 — close icon parity

**Status: Implemented; owner testing deferred to the complete build.** Owner: Codex implementation / human acceptance. Branch: `feature/kindle-play-improvements`. Started/automated verification: 2026-11-09. Implementation revision: `95eeabb`.

Reuse: Play now calls the existing `draw_close_button` and `header_control_visual_rect` helpers used by Puzzles/Review. `Layout.exit`, `PlayTarget::Close` and the existing runtime exit path are unchanged. Extraction: none; the two existing helpers were made `pub(crate)` without changing their drawing bodies. New: a focused pixel-parity/X-endpoint and touch-boundary test in the existing Play render harness, because Play previously had no comparison against the other products' close chrome. No new production widget, state, geometry or network code.

1. Replace the literal “CLOSE” in Play render by calling the **existing X close-button renderer**, exposing or extracting its shared helper if necessary, with the same size, contrast and top-right visual placement as Puzzles/Review. Do not copy the X drawing code. Reuse/extract the existing close drawing/button primitive; keep Play's Close target in the same touch rectangle and retain clean shutdown. Do not change Puzzles/Review source appearance or hit geometry.
2. Add visual/snapshot and hit-target tests on 1860×2480 and 1404×1872 layouts; assert X renders, text CLOSE absent, tap hits only its configured target, and existing product goldens remain unchanged.

Automated evidence (2026-11-09, local): the new parity test first failed against the literal CLOSE button, then passed after reusing the existing helpers. `PATH="$PWD/.venv/bin:$PATH" bash scripts/codex-play.sh verify FKP-01` passed in full: Play render/touch tests (5), HTTP/runtime tests (5), pinned Play build including `scripts/check.sh`, stage and artifact verification. Existing Puzzles/Review goldens passed unchanged. Only the six Play golden checksums changed; all six close-icon crops and the ready full frame were visually inspected (`target/fkp-01-close-states.png`, `target/fkp-01-ready.png`). Verification log: `.git/codex-play-runs/` via `latest-FKP-01`.

Prepared package: `target/kindle/play/kindle_chess_play_0.1.0_kindlehf.kpkg`. Binary SHA256: `2e82dc2680eef91f919824a6d2ee0657967cabf888ad3996f97f07c77b03c965` (3088568 bytes). Toolchain/target checks passed: Rust 1.85.1, Zig 0.13.0, ARMv7 hard-float/glibc 2.35 and pinned FBInk. No device connection/deployment or human pass is claimed. [Prepared checkpoint](device/ks1-barolo.md#fkp-01--prepared-close-icon-parity-waiting-for-human-2026-11-09). FKP-02 stays Blocked until this human acceptance and desktop BKP-03 are Done.

**Automated acceptance:** focused render/runtime tests and scripts/check.sh. **Human verification (last step):** on Scribe compare Play and Puzzles close icons visually; tap Play X and confirm reliable shutdown. Wait for owner approval before Done.

### FKP-02 — choose White/Black before New Game

**Status: Implemented; final verification pending.** Owner: Codex implementation / final owner testing. Branch: `feature/kindle-play-improvements`. Started: 2026-11-09.

Reuse: existing Play state, worker/client, UCI/FEN validation, board geometry, board painter and button selection chrome. Extracted the existing canonical/display-square transform into `eink_ui::board::oriented_square` for board drawing, selection highlights and taps. New: side-selection target/state and color-neutral request/turn checks, because Play previously represented White only. Focused tests cover Black opening ply 1, White legacy snapshots, both orientations/all 64 squares, promotions, error/reconnect color and canonical move requests. White/Black previews were inspected. Full matrix verification follows after FKP-03 under the owner override.

1. Add a clear side selector (WHITE / BLACK) beside pre-game Elo controls, using existing button/selection visuals and Play's control layout rather than a new widget framework. Default White. Only editable when New Game is permitted; selecting after game start must not mutate the active session. Request::New sends {rating,playerColor}; retain a legacy White path only where appropriate. On GET reconnect, use snapshot.playerColor as source of truth rather than cached selection.
2. Update Snapshot validation: accept both color strings; active side follows FEN and ply, while nonterminal states end **on the human turn** (White => even ply; Black => odd ply). Retain authoritative FEN, UCI history replay/consistency checks, terminal parity exception and error semantics; avoid simplistic White-only assumptions. Human Black 201 may contain one Maia opening move and must render that board immediately.
3. Rotate the **existing shared board renderer** 180 degrees for Black, with correct file/rank labels, last-move highlighting (if any), and inverse touch mapping from on-screen squares to canonical board indices. Reuse/extract a shared orientation mapping helper for rendering, highlighting and hit tests; do not build a separate Black board. Only the human's pieces can be selected; Black promotion happens at the opposite edge from White. Both ordinary and promotion UCI remain canonical standard coordinates. Disable input during requests/errors/finished states. Update status copy to say “Black to move”/“White to move” based on state.
4. Tests first: new-game JSON, legacy responses, Black 201 ply=1, White normal ply=0, both orientation layouts + all 64 touch squares, Black promotion/castling, unexpected response, 409/422 and reconnect showing server-chosen color. Do not change shared board model or make Kindle decide chess legality.

**Automated acceptance:** core + HTTP + render + runtime focused tests, scripts/check.sh; pinned Play build/stage/verify if runtime artifacts changed. **Human verification (last step):** on Scribe start and complete two moves as White and Black with real desktop Maia; confirm Black board orientation, taps, status and no extra opening move; reconnect and verify color preserved. Owner approval gates Done.

### FKP-03 — paged move-history panel and read-only navigation

**Status: Implemented; final verification pending.** Owner: Codex implementation / final owner testing. Branch: `feature/kindle-play-improvements`. Started: 2026-11-09.

Reuse: extracted the original Snapshot replay loop into `replay_positions`; validation and all browsing use that single castling/en-passant/promotion path. Added a generic `Board::from_squares` constructor without legality logic. Extracted existing analysis `draw_move_chip`, `draw_page_control` and shared `PageWindow` pagination into neutral `eink_ui::move_list`; the original analysis still uses those identical painters. Play opts into a shared board-height limit to reserve legible history space. New product-owned state/targets track view ply and page, preserve latest FEN/history, block writes in the past, and restore exact authoritative state with Latest. No fake Review/Puzzles trees or persistence.

Focused core tests (12), Play render tests (8) and HTTP/runtime tests (8) pass; Clippy passed before the final focused additions and will run in the full gate. Covered 0/1/2/108 plies, SAN/UCI labels, both colors/orientations, Start/middle/latest, all pagination boundaries, all special-move fixtures, finished browsing, pending requests, no HTTP writes from browsing, reconnect-to-latest, pixel-damage reconstruction and legacy/additive wire data. Twelve history goldens at both resolutions were visually inspected; all older product golden files remain untouched.

1. Rework the **existing** Play status area (currently a text box) to retain a clearly readable network/game-status message **and** show all committed moves in an e-ink-friendly, paginated move list. Reuse the neutral presentation, font, hit-test and pagination primitives behind the Puzzles/Review analysis panel where feasible; do not create fake analysis-tree data to force reuse. Reuse typography, border and controls consistent with the Puzzles/Review analysis panel; do not import those products' persisted trees or states. Display move numbers and SAN from optional sanMoves; if missing, fall back to UCI. Black's initial Maia White move is immediately visible. Highlight selected ply and expose page navigation; do not truncate older moves silently.
2. Add Play-only hit targets: HistoryPly(n), HistoryPreviousPage, HistoryNextPage, Latest. Tapping a move displays **the position after that ply**, from the server-authoritative starting position and UCI sequence. **Refactor and reuse the existing replay loop in `chess-core/src/play.rs` `Snapshot::parse`** to obtain intermediate positions (castling, en passant and q/r/b/n promotions); keep one replay implementation for both validation and history rather than writing a second one. No on-device legality/search/PGN parsing. A tap on the current last move or **Latest** displays the exact snapshot.fen; the initial position is reachable (Start/ply 0).
3. Keep immutable latest snapshot (FEN/history/ply) separate from viewPly and display board. While viewing the past, block move entry, promotion, resign/other mutating board actions as appropriate; never send an HTTP move based on viewPly. Make **LATEST / PLAY** button obvious and enable input only after returning to live end. Preserve current server expectedPly for future moves. On new snapshot/reconnect, clear stale selection and return to Latest; errors and pending requests never fabricate history.
4. Paginate deterministically for short and long games, maintain safe  e-ink touch regions and avoid overlap with toolbar, color controls, status text, or promotion modal. Verify normal Scribe and smaller display sizes, full refresh/damage correctness, and no regressions to Puzzles/Review snapshots.
5. Tests first: 0/1/2/100+ ply, Black starts at ply 1, both SAN/UCI labels, tapping first/middle/latest, Start, page boundaries, parity, en passant, both castles, q/r/b/n promotion, old-vs-live board selection, no network write on history navigation, restoring exact canonical FEN on Latest, finished games browseable.

**Automated acceptance:** focused core/render/runtime HTTP tests; scripts/check.sh, Play build/stage/verify where appropriate. **Human verification (last step):** on physical Scribe play 6+ plies, tap early move and verify board/selection/highlight, page through moves, tap Latest, make next live move, disconnect/reconnect and confirm preserved server sequence. Owner approval gates Done.

### FKP-04 — final cross-repository acceptance (human is the last step)

**Status: In progress (host verification; owner tests once the complete build is ready).** Owner: Codex test prep / final owner acceptance. Branch: `feature/kindle-play-improvements`. Started: 2026-11-09.

1. Codex prepares automated verification only: desktop npm test, npm run build, npm run test:desktop; Kindle focused tests and scripts/check.sh; pinned scripts/app.sh build play, scripts/app.sh stage play, python3 tools/verify_app_artifacts.py --app play; confirm old two products still pass check.sh. Record exact actual pass/fail evidence and both source SHAs. This stage may be marked Waiting for human, **not Done**.
2. Owner on real desktop + jailbroken Kindle Scribe on trusted Wi-Fi: manually Start/Stop in Maia tab; play both colors and at least two Elo settings; verify GUI game/last-move time; make several moves, leave Kindle, confirm matching dated/rated PGN in My Studies and update after next move; tap old plies and Latest on Scribe; verify resign result for each color, reconnect after timeout, and stop/restart listener. Confirm no writes to other Kindle apps, Games or Openings data, and no loss of user-edited studies. Record firmware, app binary/source revisions, dates, what was actually observed and any failed steps in docs/device/ks1-barolo.md.
3. **Human verification is the final gate:** request explicit owner approval based on that record; only then mark FKP-04 Done in both status locations and declare the cross-repo improvement set accepted. Do not claim a device pass from automated tests, remote CI, or mock Maia.

## How to hand the next task to Codex

“Read docs/KINDLE_PLAY_IMPROVEMENTS.md, AGENTS.md and the linked backend contract. Implement only the first Ready FKP task, tests first; update table and detailed status/evidence together; stop after automated tests at Waiting for human. Do not start dependent tasks before owner verification.”

The MVP is already accepted. This is a new queue and does not reopen PLAY-01..05 or the unrelated DATA-01/02 migration.


## Testing the complete feature build

Choose WHITE or BLACK and Elo before NEW GAME. Black waits for the desktop's
single White opening move and starts with the rotated board; no local Maia move
is generated. The desktop must implement the additive v1 color extension.
Snapshots without `sanMoves` still work and display standard UCI instead.

Tap any move to view the position after its ply, or START for the initial board.
Use the numbered arrows to page through the entire record. While viewing the
past, board input, promotions, resign and new-game mutations are disabled.
LATEST / PLAY returns to the exact current server snapshot and enables live
moves again. A server update/reconnect returns to Latest automatically.

The feature branch keeps all changes together under the owner's 2026-11-09
override. The complete staged package will be handed over once automatic checks
finish; owner testing happens once, after all Kindle implementation is ready.

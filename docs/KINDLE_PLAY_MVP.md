# Kindle Chess Play — third app MVP

Status: **In progress**. Started: 2026-11-09.
Repository: `abavelski/kindle-chess-puzzles`; default branch **main**.
Goal: add a **third independently packaged Kindle Scribe app**, **Kindle Chess Play**, to play White against Maia on a desktop reachable over trusted local Wi-Fi. This plan is standalone: the Kindle agent can implement everything with a fake HTTP server without checking out the desktop repo.

## Current code and constraints

- Existing standalone entrypoints: `apps/kindle-puzzles`, `apps/kindle-game-review`; workspace in root `Cargo.toml`. Reuse shared `chess-model`, `eink-ui`, `chess-render`, `kindle-platform`, `kindle-app` and runtime `crates/kindle-app/src/runtime.rs` (`Application::poll` supports asynchronous completion).
- `chess-model::Board` represents pieces/FEN and tap/promotion staging, **not move legality**. The desktop server validates chess moves and owns the authoritative FEN. Kindle renders a new board only on successful server response, not on tap. Use `UciMove` and existing square conversion, including promotion picker. No engine, chessops, PGN parser or move legality implementation on Kindle.
- Existing rendering `ScreenState` and `chess-render/src/layout.rs` are biased toward Puzzles/Review. Extend with a **separate Play screen path/state and Play hit targets** reusing board, pieces, Gray8 canvas, button components, damage-aware refresh and shared lifecycle, rather than inventing fake puzzle/review data or copying the old combined runtime. Keep existing Puzzles/Review layout/snapshots unchanged. Do not add a Play case to the persisted Puzzles/Review `Workspace` settings schema just to render the app.
- Existing packages deliberately isolate runtime roots/locks, with **global shared framebuffer owner**: keep the supervisor's global exclusivity and shutdown cleanup. Do not copy Puzzles' optional Chessnut worker. Device baseline: jailbroken Scribe Barolo / ARMv7 / glibc 2.35 (see `docs/BUILD_DEPLOY.md`).
- The unfinished `docs/DATA_SEPARATION_PLAN.md` (DATA-01/02) concerns legacy Review files under Puzzles. **This Play app is entirely independent of that migration**: never read/write Review games, puzzle progress, either app's settings, resume or legacy fallbacks. No dependency on those tasks completing first.

## MVP screen / flow

1. First launch reads a tiny own-app config file `/mnt/us/kindle-chess-play/config.json`, e.g. `{"serverUrl":"http://192.168.1.23:8765"}` (desktop's trusted-LAN IPv4). **No touchscreen IP keyboard** in v1: edit the file over SSH/USB; show a helpful "Configure serverUrl" message if absent/invalid. Validate scheme `http`, host, port and length; do not send HTTP to arbitrary user-supplied paths.
2. Main Play screen shows **Maia rating** (600–2600 in steps of 100, default 1500; simple `-`/`+` controls), **New Game**, **Refresh/Retry**, **Resign**, **Close**, board, last move and clear status text. Reuse existing sizable touch targets and e-ink refresh policies. Fix White as player/orientation in v1; no Flip/Free Board/analysis UI.
3. On open, GET health then GET current game. If desktop still holds an active game, show that game at its authoritative FEN/history. If no game, show a starting board and New Game. Rating may be changed only **before** starting another game.
4. New Game POST creates a game; show the returned initial board. Taps select a White piece and destination; on promotion present existing q/r/b/n chooser. Convert taps to standard UCI, **do not change displayed piece positions** until server confirms. While the move is in flight, show `Maia thinking…` and ignore second move taps. A completed 200 response replaces displayed state, including **both** the White and Maia moves. A 422 leaves the board untouched and displays "Illegal move"; 409 receives authoritative state and prompts the user to retry selection.
5. Resign POST ends active game with a finished/result message; New Game can then replace it. On checkmate, draw or resignation, disable board moves. Close always exits cleanly. On network timeout/disconnect or late failed response, show a useful error and offer **Refresh/Retry via GET**, never blindly resubmit a move. Reconnection during the same desktop process restores state; if desktop restarted, return to No game.

## Kindle-only configuration / footprint

- New Cargo binary: `apps/kindle-chess-play`, proposed binary name `kindle-chess-play`, with marker `KINDLE_CHESS_PRODUCT:play:v1` (match existing product-identity checks).
- New product adapter: `crates/kindle-app/src/products/play.rs` with its own `PlayApp`, `PlayRuntime` and `run_play()`; separate pure state logic (in `chess-core` or new product-specific module) and HTTP adapter. Only the Play app accesses the network. A small blocking HTTP client on a **dedicated worker thread** with `mpsc` messages is sufficient; no async runtime or TLS stack required for plain HTTP on trusted LAN. Use bounded request size and parse/validate server JSON. Keep renderer/input thread non-blocking even when Maia cold-loads; timeouts longer than desktop's engine deadline.
- Distinct root **`/mnt/us/kindle-chess-play`**, own `config.json`, logs, scriptlet, lock, package ID `kindle_chess_play`, manifest and installed binary. Prefer no save-game file for v1: GET server current on launch; config only persists. Preserve bytes on malformed/future config rather than silently overwriting them.
- Extend `tools/kindle_build.py` product table, `scripts/app.sh` selector, `tools/verify_app_artifacts.py`/CI matrix and packaging to include `play`; keep existing Puzzles/Review product artifacts byte-identifiable and do not add new Play data to their packages. Use existing `scripts/app.sh build play`, `stage play`, and verification pattern. Existing scripts already enforce pinned ARMv7/glibc requirements.

## Frozen MVP HTTP contract (copy held in this repository)

**Implement this contract exactly; do not require the other repository at build/test time.** The desktop is the authoritative chess rules engine and position store. The Kindle is a thin HTTP client. HTTP/1.1, UTF-8 JSON, `Content-Type: application/json`, base URL `http://<desktop-LAN-IP>:8765` (configurable host/port). Routes use the exact spelling below. One active game maximum, one human player **White**, Maia plays **Black**. A game is held **in memory** on the desktop; there is no database, authentication, multiplayer, or persistence in v1. The currently finished game remains readable until a new one is started.

| Method | Route | Request | Success |
| --- | --- | --- | --- |
| GET | `/api/v1/health` | none | 200 `{"apiVersion":1,"maiaAvailable":true}` (availability may be false) |
| POST | `/api/v1/games` | `{"rating":1500}` | 201 game state; replaces a **finished** game only; 409 if one is playing |
| GET | `/api/v1/games/current` | none | 200 game state; 404 `NO_GAME` if no game |
| POST | `/api/v1/games/current/moves` | `{"uci":"e2e4","expectedPly":0}` | 200 game state **after** human move and Maia reply, unless human move ended game |
| POST | `/api/v1/games/current/resign` | `{}` | 200 finished state (White resigns, Black wins) |

- Only integer ratings **600, 700, ..., 2600** are accepted (same range/steps as the desktop Maia selector); default UI choice 1500. Unsupported values: 400 `BAD_REQUEST`.
- Game state is the same JSON shape in every successful game response:

    {
      "apiVersion": 1,
      "rating": 1500,
      "playerColor": "white",
      "fen": "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      "moves": [],
      "ply": 0,
      "status": "playing",
      "result": null,
      "lastMove": null
    }

- `fen` is a complete six-field FEN for the authoritative current position. `moves` is the whole ordered, standard-UCI move history from the initial chess position; `ply === moves.length`; `lastMove` is the last UCI move or null. Use normal king-destination castle moves (e.g. `e1g1`, **not** king-to-rook), promotions `e7e8q`/`r`/`b`/`n`. `status` is `playing` or `finished`. When finished, `result` is `{"outcome":"white_win"|"black_win"|"draw","reason":"checkmate"|"stalemate"|"draw"|"resignation"}` with actual scalar strings; when playing it is null. The API does not expose pending Black-to-move half-states.
- **Move transaction:** reject malformed UCI (400), illegal human move (422 `ILLEGAL_MOVE`), wrong `expectedPly` (409 `STALE_PLY`), finished game (409 `GAME_FINISHED`), or another ongoing request (409 `BUSY`). The server verifies the move against chess rules, tentatively applies it, checks game end, otherwise gets exactly one Maia `bestmove`, verifies/applies the legal reply, and only then atomically publishes the resulting state. On Maia failure/timeout (503 `ENGINE_UNAVAILABLE` or 504 `ENGINE_TIMEOUT`), **leave the prior game state unchanged**.
- Each error returns `{"error":{"code":"CODE","message":"Readable explanation"}}`; game-specific errors **also** include a top-level `"state"` holding the unchanged/current authoritative game state, if one exists. Reject oversize requests (max 4 KiB JSON), unsupported methods and unknown routes with appropriate HTTP errors. No CORS is needed.
- A delayed/lost POST response is ambiguous to the client. **Before retrying a timed-out move, GET current state**; inspect `ply`/`moves` to decide if it committed. `expectedPly` makes an accidental repeat a conflict instead of a second move. Kindle must never locally commit a move before server confirmation.
- GET/POST are synchronous request-response APIs, **not** streaming or polling for Maia completion. Each move POST waits for Maia; the client must use a background worker, a visible "Maia thinking" state and a sufficiently long timeout for cold model loading (e.g. ~195 s, with desktop engine deadline shorter). No move requests while one is pending.
- Game completion: finish on checkmate, stalemate, applicable automatic draws, or explicit resignation. There is no pause/resume protocol: GET current lets the Kindle reconnect while the same desktop process runs; desktop restart loses the in-memory game. One game per server process at a time is sufficient for MVP.

### Contract checks both repositories must pass

1. Start rating 1500 → standard starting FEN, `ply=0`, White; GET returns identical state.
2. Legal `e2e4` at `expectedPly=0` → 200, server supplies one **legal Maia** Black move, `ply=2`, complete updated FEN and history.
3. Illegal `e2e5` → 422, board/history/ply unchanged; stale `expectedPly` → 409, never creates a duplicate move.
4. Missing server, Maia unavailable, timeout or disconnect → readable error, no optimistic Kindle board change; reconnect via GET.
5. Resign → `finished` with `black_win` / `resignation`; reject further moves; a new game may replace finished one.
6. Promotion and castling UCI/FEN handling are standard and verified with focused tests even if reaching them through a full on-device match is unlikely.

## Kindle implementation checklist (task tracker)

| Task | Depends on | Status | Deliverable |
| --- | --- | --- | --- |
| PLAY-01 | — | Done | Pure Play state machine + mockable HTTP client/contract fixtures |
| PLAY-02 | PLAY-01 | Done | Product-specific Play board, touch targets and status screens |
| PLAY-03 | PLAY-01, PLAY-02 | Done | Kindle PlayRuntime wiring, worker-thread requests, reconnect |
| PLAY-04 | PLAY-03 | Ready | Third independent Cargo binary, staging/install/release matrix |
| PLAY-05 | PLAY-04 | Blocked | Host regressions, pinned build and physical Scribe LAN acceptance |

### PLAY-01 — state/client tests first

Status: **Done**. Owner: Codex. Branch: `feature/kindle-chess-play-mvp`. Started/completed: 2026-11-09.

Evidence: tests first failed with missing Play modules, then `cargo test -p chess-core --test play` (5 tests) and `cargo test -p kindle-app --test play_http` (2 tests) passed. `PATH="$PWD/.venv/bin:$PATH" scripts/check.sh` passed, including both existing app targets and unchanged golden frames. Fixtures cover both castles, q/r/b/n promotions and en passant; snapshot validation reconstructs recorded placement without deciding legality. Network tests use local HTTP responses for refusal, delay, 409/422/503/504, malformed/oversize bodies and reconnect GET. No device deployment.

- Add product-owned state: rating (100-Elo steps), server state snapshot, selected piece/promotion, `idle|connecting|waiting|playing|error|finished` UI status and in-flight request guard. No puzzle/review model dependency or writes.
- Implement minimal client for `GET /health`, `POST /games`, `GET /games/current`, `POST /moves`, `POST /resign`. Use endpoint paths from frozen contract above. Parse full six-field FEN, `moves`, `ply`, `lastMove`, result and `error.code`; safely reject oversized/malformed JSON. Validate `moves.length === ply` and consistency with the board. No optimistic mutation.
- Start with a local test HTTP server/fake responses in Rust host tests; include illegal/stale moves, connection refused, slow response, 503, 504, duplicate tap suppression, malformed state, and reconnect GET. Mock network at a boundary so core tests are fast. No Maia dependency.

### PLAY-02 — Play-specific e-ink UI

Status: **Done**. Owner: Codex. Branch: `feature/kindle-chess-play-mvp`. Started: 2026-11-09.

- Create a Play screen/view inside `chess-render` that delegates board, fonts, buttons and Gray8 drawing to existing reusable code; add Play-only `HitTarget` mappings. Avoid using authored puzzle data as fake game data and avoid changing the Puzzles/Review UI to expose Play controls.
- Render White board from server FEN, selected piece, pending promotion chooser, rating +/- and New Game/Resign/Refresh/Close, result/status/errors. Disable interactions appropriately while waiting or after end. Status must remain legible on e-ink; refresh damage on both changed squares and status; full refresh still available.
- Host layout/hit-test tests at standard and small Scribe display dimensions; snapshots for disconnected, ready, playing, waiting, illegal move and finished. Preserve all old product golden frames.

Evidence (2026-11-09): `cargo test -p chess-render --test play` first failed on missing Play view/targets, then passed all 4 tests. Checked all 64 White square targets at 1860×2480 and 1404×1872, disabled controls, promotion modal, finished input, pixel damage reconstruction and six visually inspected Gray8 snapshots (`target/play-*.pgm`). `PATH="$PWD/.venv/bin:$PATH" scripts/check.sh` passed, including existing product goldens. No device deployment.

### PLAY-03 — PlayRuntime and device boundary

Status: **Done**. Owner: Codex. Branch: `feature/kindle-chess-play-mvp`. Started: 2026-11-09.

- Use `kindle_app::runtime::Application` shared foreground framebuffer/input/power loop and supervisor lock. Worker owns socket/timeouts; screen thread checks completion in `poll` and re-renders. No blocking TCP during touch/power processing.
- On startup load **only** Play's `config.json`; if valid, health/current GET. On move, send `expectedPly` from last server snapshot; after 200 replace FEN and history atomically. On 409 apply `state`; on 422 keep board + show rejection; after timeout show "Refresh server state" rather than automatically replaying POST.
- Handle cancel selection, promotions q/r/b/n, Black reply update, checkmate/draw/resign and clean close. Test the worker against a mock server, including a server response after user sleeps/wakes; avoid stale UI overwrite (per-request sequence/generation).
- No modifications to DATA-01/02 migration, legacy settings fallbacks, or Puzzles/Review persistence.

Evidence (2026-11-09): `cargo test -p kindle-app --test play_runtime` failed before the adapter existed, then passed 3 tests with a controllable fake client and real loopback HTTP server. Covered nonblocking health/current/move completion, delayed completion over sleep, failure followed by GET reconciliation without POST replay, immediate close during a blocked request, config reload and preservation of malformed/future bytes. `PATH="$PWD/.venv/bin:$PATH" scripts/check.sh` passed. RETRY reloads config and fetches health/current; REFRESH retains the shared full-screen redraw. Pinned third-product builds follow in PLAY-04/05; no device deployment.

### PLAY-04 — independent app packaging

- Add workspace member and product entrypoint; expose `run_play()`. Update only the product selector/packaging/release logic required for the third product. Separate package/app identity, scriptlet, data root, locks and logging, but share the global display owner as before.
- Add Play manifest and launch/install/uninstall template handling. Update artifact verifier/test scripts/CI matrix to build/stage/verify all **three** independently. Verify no game/puzzle library or Chessnut binary is included in Play; verify existing two build artifacts still pass their checks.
- Add a short operator section here explaining how to set `serverUrl` over SSH or USB and launch each app (no new configuration UI needed).

### PLAY-05 — validation and human checkpoint

- Run focused Rust state/client/UI tests then `scripts/check.sh`; build/stage/verify `puzzles`, `review`, and `play` using the pinned Scribe target. Record commands and results, and ensure old snapshot tests remain green.
- With a physical jailbroken Scribe + desktop server on same trusted Wi-Fi, install Play and check: configure server URL; NEW game at two ratings; legal white move and Maia response; illegal move unchanged; promotion chooser via fixture or test position; disconnect/reconnect GET; resignation; finish message; close; and distinct co-installation with the other apps. Confirm input exclusivity and no extra files created in either older app's data root.
- Append actual deployment/firmware/observations to `docs/device/ks1-barolo.md`. Until the device run actually succeeds, leave human acceptance at **Waiting for device**. Update table and detailed statuses together per `AGENTS.md`.

## Done when

The third app installs and runs separately on Kindle Scribe without reading/writing any Puzzle or Review data. It starts an Elo-selected game, submits White's standard UCI move to the LAN server, shows Maia's **validated** reply from server FEN, rejects invalid moves without desynchronization, recovers by GET after connection problems, and ends on checkmate/draw/resign. Existing two apps and pending data separation tasks remain untouched.

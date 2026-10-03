# Phase-one reference parity

Task 09 closes the first Kindle milestone against the **currently implemented** behavior of
`abavelski/eink-chess-app`. It does not expand the product beyond the compatibility
contract in [PUZZLE_FORMAT.md](PUZZLE_FORMAT.md).

The evidence below combines host tests, deterministic Gray8 snapshots, and the recorded
physical Kindle Scribe checkpoints in [device/ks1-barolo.md](device/ks1-barolo.md).
A row marked **Pass** means the required behavior is covered at the layer where it can be
verified. Hardware-only observations are not inferred from host tests.

## Parity matrix

| Behavior | Automated evidence | Visible/render evidence | Physical Scribe evidence | Status |
| --- | --- | --- | --- | --- |
| v1 collection parser/validation | `core_compat`: reference fixture parsing plus invalid FEN/IDs/version/solutions | Invalid collection is rendered in the FILES picker | Checkpoint D loaded valid collections and preserved/reported a malformed collection | **Pass** |
| 256 KiB compatibility limit | `core_compat::puzzle_parser_rejects_invalid_fen_empty_collection_and_oversize_file` | Not separately visible | Hardware-independent parser policy | **Pass** |
| FEN side-to-move/orientation | `core_compat::fen_parses_piece_placement_and_active_color`; `parity_vectors` checks side and initial orientation | White/black orientation snapshots | Checkpoint C verified automatic Black-facing and return to White-facing orientation | **Pass** |
| exact UCI solution matching | `core_compat` UCI round-trip/rejection; `app_state` grades stored moves exactly | Wrong/correct snapshots | Checkpoint C verified a deliberately wrong promotion choice rolls back before the exact stored underpromotion succeeds | **Pass** |
| wrong rollback | `app_state::selection_deselection_and_wrong_moves_do_not_advance_solution` | wrong snapshot/parity flow | Checkpoint C: d7-d8 and wrong Q promotion rolled back | **Pass** |
| automatic replies | `app_state::three_ply_line_auto_applies_coordinate_reply_then_accepts_final_move`; shared parity vectors | correct/intermediate parity snapshot | Checkpoint C verified e2-e6 -> automatic f7-f8 | **Pass** |
| solver completion lockout | `app_state::one_move_solution_completes_marks_solved_once_and_ignores_more_board_taps` | complete/solved snapshots | Checkpoint C verified extra board taps do not change the completed position | **Pass** |
| promotion q/r/b/n | `parity_vectors::application_grades_all_four_promotion_choices_exactly`; board-layer q/r/b/n contract | promotion chooser snapshot | Checkpoint C verified the Q/R/B/N chooser and physical Q, N, and R promotion paths | **Pass** |
| automatic opponent promotion | `app_state::automatic_opponent_promotion_applies_suffix_without_opening_promotion_state`; shared vector | parity flow | Checkpoint C verified a2-a1=Q is automatic and opens no chooser | **Pass** |
| Solution/Free Board transition rules | `app_state::free_board_is_ungraded_and_mode_transitions_preserve_the_required_state` | Free Board/parity snapshots | Checkpoint C verified ungraded arbitrary move, reset, and return to Solution | **Pass** |
| reset | app-state reset/navigation tests | solved/reset parity states | Checkpoint C verified original FEN restoration and transient-state clearing | **Pass** |
| previous/next no-wrap | `app_state::navigation_availability_matches_collection_ends_without_wrap` | disabled-end parity snapshot/layout | Checkpoint C verified Previous/Next disabled and inert at collection ends | **Pass** |
| flip | orientation app-state tests | orientation parity snapshots | Checkpoint C verified manual Flip | **Pass** |
| orientation lock | `app_state::orientation_lock_flip_unlock_and_navigation_follow_reference_rules` | orientation-lock snapshot | Checkpoint C verified locked orientation across navigation and resumed auto-orientation after unlock | **Pass** |
| description auto-reveal/manual toggle | `app_state::description_toggle_does_not_solve_and_completion_reveals_it` | hidden/visible/long-description snapshots | Checkpoint C verified pre-solve toggle, auto-reveal, and post-solve hide | **Pass** |
| difficulty string/number | `core_compat::collection_title_description_difficulty_and_side_to_move_match_v1` | string and numeric difficulty snapshots | Presentation is exercised by the same Scribe header layout | **Pass** |
| result feedback | app-state Wrong/Correct/Complete transitions | wrong/correct/complete snapshots | Checkpoint C repeatedly observed Wrong, Correct, and Complete feedback | **Pass** |
| Sashité rendering | deterministic generated-asset and render tests in the normal check gate | board snapshots use generated Sashité assets | Checkpoint C: pieces legible at normal reading distance | **Pass** |
| multiple collection discovery/picker/error behavior | `kindle-platform/tests/storage.rs`; app picker/pagination tests | picker, picker-error, warning snapshots | Checkpoint D verified switching, computer-copy discovery, malformed selection, and removal | **Pass** |
| active collection restore | app-state restart/activation tests; storage round trip | solved/picker states | Checkpoint D verified active collection after relaunch | **Pass** |
| per-file current puzzle restore | `app_state::restart_restores_active_file_current_puzzle_and_solved_ids_per_file` | current-puzzle header state | Checkpoint D verified remembered position in each collection | **Pass** |
| solved IDs/marker | progress/core + app-state completion/idempotence tests | solved marker snapshot | Checkpoints C/D verified solved marker and solved IDs across restart | **Pass** |
| progress failure/future-version protection | `kindle-platform::storage` dirty-retry and corrupt/future protection tests | progress-warning snapshot | Checkpoint D verified source collections unchanged; package update/uninstall/reinstall preserved progress | **Pass** |
| clean exit/relaunch | app Exit effect + `kindle-chess::exit_flushes_dirty_progress_and_requests_normal_return`; lifecycle contract tests | X control/modal routing snapshots | Repeated normal exit, crash recovery, library launch/exit, and progress restore passed on-device | **Pass** |

## Shared fixture vectors

The six logical example/promotion cases used by the milestone are in
[`tests/fixtures/parity-puzzles.json`](../tests/fixtures/parity-puzzles.json).
Task 09 adds
[`tests/fixtures/parity-vectors.json`](../tests/fixtures/parity-vectors.json),
which records the expected start FEN/side, solver-entered plies, automatic replies,
final board placement, completion, and solved state. The
`crates/chess-core/tests/parity_vectors.rs` test executes those vectors through
the real `AppState`.

| Fixture | Start side | Solver actions | Automatic reply | Expected final placement | Complete/solved |
| --- | --- | --- | --- | --- | --- |
| one-move | White | d7e8 | — | `4B3/6pp/p5k1/6P1/1ppp1K2/8/1P6/8` | yes / yes |
| three-ply | White | e2e6, e6f7 | f7f8 | `r1bq1k1r/pp1nbQp1/2p4p/8/2BP4/1PN3P1/P4P1P/3R1RK1` | yes / yes |
| promotion-white-queen | White | a7a8q | — | `Q6k/8/8/8/8/8/8/K7` | yes / yes |
| promotion-white-knight | White | b7b8n | — | `1N5k/8/8/8/8/8/8/K7` | yes / yes |
| promotion-black-rook | Black | h2h1r | — | `k7/8/8/8/8/8/8/K6r` | yes / yes |
| promotion-auto-reply | White | h2h3, h3h8 | a2a1q | `7R/8/8/8/8/8/8/q6K` | yes / yes |

The vector check intentionally models the product's physical-board semantics. It is not a
chess-engine legality test.

## Reliability soak and HUMAN CHECKPOINT G

Phase-one hardware work already constitutes a cumulative real-device soak rather than a
new synthetic Task 09 session:

- Task 04 logged **53 accepted finger taps** and a repeated-move path with no coordinate
  drift or misses.
- Task 05 then exercised nine full parity groups on the Scribe. The scripted groups require
  more than 50 additional taps even before navigation between cases, so the cumulative
  milestone evidence exceeds the Task 09 minimum of 100 physical touch interactions.
- Task 06 repeatedly browsed collections and verified active/current/solved progress across
  restart.
- Task 07 repeated navigation, NOTE, promotion/cancel, normal exit, SIGTERM, and killed-child
  recovery. A first refresh implementation was too slow; the batched release was then
  measured and accepted as responsive with clean redraws. Exclusive finger input fixed the
  observed native-menu/duplicate-writer contention.
- Task 08 repeatedly launched/exited the installed package, verified the final display
  ownership handoff, and preserved puzzle/progress hashes through update and
  uninstall/reinstall.

The resulting checkpoint-G evidence is: parity behavior passes; normal restart preserves
progress; repeated coordinate/input checks do not show frequent failures; non-sleep
ghosting/refresh behavior was accepted after the batching fix; launch/exit/recovery is
repeatable; and the library Scriptlet provides a child-facing launch/exit path without a
terminal.

### Explicitly unverified lifecycle cases

Two lifecycle observations are deliberately **not** promoted into passes:

1. natural idle suspend/resume remains unverified; Task 07 was closed with that check
   deferred by the user, and no resume support is claimed;
2. library launch after a full device reboot remains unverified; ordinary library
   launch/interaction/X-exit and native recovery are verified.

These are known phase-one validation gaps, not hidden parity failures. They should be
retested if later real-user feedback indicates a suspend/reboot problem.

## Deferred product ideas

These were old roadmap ideas, not current-reference parity, and remain deferred to a later
product phase:

- **Unsolved only** navigation;
- **version-2 branching/rich solutions**.

Statistics, stylus-specific product interactions, engine behavior, and a Kobo platform
adapter likewise remain post-phase-one work.

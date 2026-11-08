# App-split golden compatibility baseline

Frozen on 2026-11-08 before production extraction. Existing reviewed checksum
constants stay authoritative; do not regenerate them to accommodate the split.
`solved-progress.json` freezes the exact compact version-1 serialized progress.

Run `scripts/check.sh` (with `requirements-tools.txt` installed). The named cases
below are mandatory baseline coverage, not evidence of hardware behavior:

| Contract | Executable baseline |
| --- | --- |
| Two independent restored sessions; both persistence directions | `chess-core/tests/split_baseline.rs::split_golden_two_restored_sessions_keep_progress_and_resume_independent` |
| Grading, rollback, opponent reply, promotion chooser/cancel, underpromotion, FREE | `chess-render/tests/parity_flows.rs::task05_action_state_render_flows_match_reviewed_snapshots` |
| Explicit analysis references and grayscale snapshots | `chess-render/tests/analysis_rendering.rs::analysis_visual_states_match_reviewed_gray8_snapshots` |
| Review main line, variation, FREE; STANDARD/SMALL | `chess-render/tests/review_rendering.rs::review_workspace_visual_states_match_reviewed_gray8_snapshots` |
| Full-frame damage reconstruction at both board sizes | `chess-render/tests/review_import.rs` |
| Legacy settings migration; independent profiles after restart | `chess-core/tests/settings.rs::legacy_preferences_initialize_both_workspaces_and_distinct_profiles_survive_restart` |
| Physical board input, promotion and automatic reply under every seat | `chess-core/tests/physical_board_seat.rs::all_seats_round_trip_solver_input_promotion_and_automatic_reply_without_changing_logical_board` |
| Chessnut state grayscale snapshots | `chess-render/tests/chessnut.rs::board_input_states_keep_puzzle_text_visible_in_monochrome_snapshots` |
| Sleep overlay and pixel preservation | `chess-render/tests/sleeping.rs::sleeping_overlay_preserves_board_outside_box_and_progress` |
| Exit, signals, crash and stock UI recovery | `tests/test_lifecycle.py` and runtime unit tests |
| Stable legacy package/root/scriptlet identity | `tests/test_split_baseline.py` |

The initial Review selector must continue to reject a renamed combined app until
SPLIT-10 replaces that guard with independent validated artifacts. Physical reply
motion and STOP remain unverified/ineffective as recorded in PHYSICAL_BOARD.md.

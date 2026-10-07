# Phase 3 — Game review tasks

Tasks 41–49 implement the planned game-review workspace described in [`docs/GAME_REVIEW.md`](../../docs/GAME_REVIEW.md).

All implementation work stays on the dedicated `game-review` branch until the user explicitly asks to integrate it into `main`.

| Task | Outcome | Status | Physical Scribe required? |
| --- | --- | --- | --- |
| [41](41-review-data-contract.md) | Freeze review JSON contract and fixtures | Ready | No |
| [42](42-pgn-review-converter.md) | Deterministic PGN -> review JSON converter | Ready | No |
| [43](43-core-review-model.md) | Review model + shared analysis parsing | Ready | No |
| [44](44-review-workspace-state.md) | Workspace/review state machine + main-line/free navigation | Ready | No |
| [45](45-review-rendering.md) | Review header, controls, movetext, SMALL-board layout | Ready | No |
| [46](46-review-hit-testing-and-picker.md) | Game picker and review hit testing | Ready | No |
| [47](47-review-storage-and-runtime.md) | Kindle game-library discovery/loading + binary wiring | Ready | No |
| [48](48-review-import-and-host-validation.md) | Import/update workflow + end-to-end host validation | Ready | No |
| [49](49-review-device-validation.md) | Physical Scribe acceptance and phase closure | Ready | **Yes** |

Use the same red -> green -> refactor discipline as the existing task series. Do not implement later tasks opportunistically.

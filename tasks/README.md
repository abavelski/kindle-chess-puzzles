# Implementation tasks

Phase one is complete and archived under [`implemented/phase-1/`](implemented/phase-1/).
Active product work is phase two: rich solution browsing. Task 10 is independent Scribe
display maintenance and is not a phase-two prerequisite.

Read `AGENTS.md`, `docs/PHASE_2.md`, `docs/PUZZLE_FORMAT.md`, then exactly one task file.

## Phase 2

| Task | Outcome | Status | Suggested fit |
| --- | --- | --- | --- |
| [20](phase-2/20-rich-analysis-contract.md) | Freeze rich-analysis JSON contract and fixtures | Ready | local model or Codex |
| [21](phase-2/21-pgn-converter.md) | Deterministic PGN -> JSON conversion | Ready | local model or Codex |
| [22](phase-2/22-core-analysis-model.md) | Parse/validate analysis trees without breaking v1 | Ready | Codex |
| [23](phase-2/23-analysis-browser-state.md) | Pure preview/navigation state, no progress mutation | Ready | Codex |
| [24](phase-2/24-analysis-rendering.md) | Paginated monochrome analysis rendering | Ready | Codex |
| [25](phase-2/25-analysis-hit-testing.md) | Tap move tokens and preview their positions | Ready | Codex |
| [26](phase-2/26-stylus-taps.md) | Map Scribe pen taps to logical UI taps | Ready | Codex + device |
| [27](phase-2/27-collection-update-workflow.md) | Stable-ID regeneration/revision/update workflow | Ready | local model or Codex |
| [28](phase-2/28-phase2-validation.md) | End-to-end host/device validation | Ready | Codex + device |

## Independent maintenance

| Task | Outcome | Status |
| --- | --- | --- |
| [10](10-scribe-ghosting-follow-up.md) | Diagnose/reduce residual Scribe panel ghosting | Ready |

Do not combine Task 10 experiments with phase-two feature changes.

## Phase-two definition of done

Keep phase-one behavior green; start host-testable behavior with a failing test; keep PGN/engine
logic off the Kindle; preserve legacy `solution` grading; keep analysis preview out of durable
progress; add physical checkpoints only where device behavior cannot be proven on the host.

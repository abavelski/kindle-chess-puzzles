# Implementation tasks

Phase one is complete and archived under [`implemented/phase-1/`](implemented/phase-1/).
Phase two is complete and archived under [`implemented/phase-2/`](implemented/phase-2/).
Post-phase-two improvements are tracked under `improvements/`.
[Task 29 — compact analysis toolbar toggle](improvements/29-analysis-toolbar-toggle.md)
moves analysis into the first icon-only toolbar control.
[Task 30 — inline PGN analysis](improvements/30-inline-pgn-analysis.md) replaces
vertical analysis-tree rows with conventional compact movetext. The former Task 10 ghosting
investigation is archived: the top-left Refresh control is the accepted manual workaround
when a residual physical-panel ghost becomes visible.

Read `AGENTS.md`, `docs/PHASE_2.md`, `docs/PUZZLE_FORMAT.md`, then exactly one task file.

## Working branch

**All Phase 2 tasks are implemented on `phase-2`, not `main`.**

Before starting any Task 20-28, verify or explicitly target the `phase-2` branch. Task commits,
tests, status changes, and related documentation stay on that branch. Do not merge or update
`main` unless the user explicitly asks for the Phase 2 integration/release.

## Phase 2 — completed and archived

| Task | Outcome | Status | Physical Scribe to complete? | Human interaction to complete? | Suggested fit |
| --- | --- | --- | --- | --- | --- |
| [20](implemented/phase-2/20-rich-analysis-contract.md) | Freeze rich-analysis JSON contract and fixtures | Implemented | No | No | local model or Codex |
| [21](implemented/phase-2/21-pgn-converter.md) | Deterministic PGN -> JSON conversion | Implemented | No | No | local model or Codex |
| [22](implemented/phase-2/22-core-analysis-model.md) | Parse/validate analysis trees without breaking v1 | Implemented | No | No | Codex |
| [23](implemented/phase-2/23-analysis-browser-state.md) | Pure preview/navigation state, no progress mutation | Implemented | No | No | Codex |
| [24](implemented/phase-2/24-analysis-rendering.md) | Paginated monochrome analysis rendering | Implemented | No | No | Codex |
| [25](implemented/phase-2/25-analysis-hit-testing.md) | Tap explicit move chips and preview their positions | Implemented | No | No | Codex |
| [26](implemented/phase-2/26-stylus-taps.md) | Map Scribe pen taps to logical UI taps | Implemented | **Yes** | **Yes — checkpoint 26A** | Codex + device |
| [27](implemented/phase-2/27-collection-update-workflow.md) | Stable-ID regeneration/revision/update workflow | Implemented | No | No | local model or Codex |
| [28](implemented/phase-2/28-phase2-validation.md) | End-to-end host/device validation | Implemented | **Yes** | **Yes — checkpoint 28A** | Codex + device |

### Validation policy

Tasks **20-25 and 27 are automated-only acceptance tasks**. An agent may implement, test,
and mark them Implemented without access to the Kindle and without asking for a human device
test. Renderer behavior in Task 24 is accepted through deterministic host snapshots; physical
readability is intentionally deferred to the final Task 28 checkpoint.

Tasks **26 and 28 cannot be marked Implemented unattended**. Their automated gates must pass
first, then the named **HUMAN CHECKPOINT** must be performed on the physical first-generation
Kindle Scribe and the observed result recorded in the repository.

Task 25 does not need a physical finger test: it owns shared hit-test geometry and action wiring.
The existing Kindle touch coordinate path is already established by phase one; real finger/stylus
interaction with the new controls is covered by Tasks 26 and 28.

## Archived maintenance

- [Task 10 — Scribe ghosting follow-up](implemented/post-phase-1/10-scribe-ghosting-follow-up.md)
  is archived with the manual top-left **Refresh** button accepted as the operational workaround.
  No additional waveform/controller investigation is currently planned.

## Phase-two definition of done

Keep phase-one behavior green; start host-testable behavior with a failing test; keep PGN/engine
logic off the Kindle; preserve legacy `solution` grading; make every interactive move visually
obvious and never infer links from plain SAN-looking prose; keep analysis preview out of durable
progress; add physical checkpoints only where device behavior cannot be proven on the host.

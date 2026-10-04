# Task 24 — Render rich analysis for e-ink

**Status:** Ready  
**Depends on:** Task 23  
**Primary area:** `crates/chess-render`  
**Validation:** Automated only  
**Physical Scribe required to complete:** No  
**Human interaction required to complete:** No  
**Completion gate:** Deterministic host rendering + snapshot tests; no device test


## Device/human-testing rule
This task must not request Kindle access. Snapshot tests are the acceptance mechanism here. Physical e-ink readability and whether the move-chip affordance feels obvious on the Scribe are deferred to HUMAN CHECKPOINT 28A.

## Outcome
Render a readable paginated main-line/variation browser while keeping the board visible.

## Required behavior
- Use the lower/status region and deterministic pages, not animation.
- Render every interactive solution-tree move and every structured prose `move_ref` as an unmistakable monochrome move chip/button: bold move text, thin rectangular outline, and visual padding.
- Plain prose, including SAN-looking text that is not a `move_ref`, must never use the move-chip treatment.
- Give the selected move chip a stronger color-independent state such as inverted fill or heavier/double outline.
- Render move chips separately from text so Task 25 can assign padded hit rectangles.
- Indent/label `main`, `alternative`, `sideline`, and selected node without color.
- Wrap long comments deterministically and keep controls reachable.
- Show analysis previous/next page controls only when needed.
- Keep phase-one snapshots unchanged when browser is closed.

## Tests
Reviewed snapshots: main-only, nested sideline, alternative label, long comment, description containing inline `move_ref` chips, unmarked SAN-looking prose staying plain, black orientation, promotion SAN, first/middle/last pages, selected move, and no-analysis state.

## Non-goals
No input dispatch or Kindle refresh tuning.

## Suggested commit
`render: add paginated rich solution browser`

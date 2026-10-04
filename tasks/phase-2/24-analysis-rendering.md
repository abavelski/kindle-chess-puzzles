# Task 24 — Render rich analysis for e-ink

**Status:** Ready  
**Depends on:** Task 23  
**Primary area:** `crates/chess-render`

## Outcome
Render a readable paginated main-line/variation browser while keeping the board visible.

## Required behavior
- Use the lower/status region and deterministic pages, not animation.
- Render move tokens separately from comments so Task 25 can assign hit rectangles.
- Indent/label `main`, `alternative`, `sideline`, and selected node without color.
- Wrap long comments deterministically and keep controls reachable.
- Show analysis previous/next page controls only when needed.
- Keep phase-one snapshots unchanged when browser is closed.

## Tests
Reviewed snapshots: main-only, nested sideline, alternative label, long comment, black orientation, promotion SAN, first/middle/last pages, selected move, no-analysis state.

## Non-goals
No input dispatch or Kindle refresh tuning.

## Suggested commit
`render: add paginated rich solution browser`

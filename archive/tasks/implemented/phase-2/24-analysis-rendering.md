# Task 24 — Render rich analysis for e-ink

**Status:** Implemented  
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


## Implementation record

Implemented on the `phase-2` branch in `crates/chess-render`.

- Added a deterministic, paginated analysis panel inside the existing lower status region, so
  the chessboard remains the primary surface and does not move when analysis opens.
- Analysis-tree moves and structured prose `move_ref` spans render as explicit bold,
  rectangular monochrome move chips. Plain SAN/UCI-looking prose remains ordinary text.
- Main, alternative, and sideline rows are labeled and nested deterministically; a selected
  node is called out explicitly and every matching move chip uses a stronger inverted
  black/white treatment.
- Long prose wraps through fixed rows and page boundaries. Previous/next controls are only
  rendered when that direction exists, while Close remains reachable on every page.
- `RenderOutput` exposes the visible move-chip and page-control rectangles as renderer-owned
  geometry for Task 25 without adding hit testing or input dispatch in this task.
- Added host coverage for explicit-vs-plain move references, selected-chip treatment,
  nested sideline indentation, first/middle/last pagination, no-analysis compatibility,
  black-to-move promotion/alternative content, and reviewed deterministic Gray8 snapshots.
  Existing phase-one rendering remains byte-for-byte unchanged when the browser is closed.
- GitHub Actions host validation passed on CI run 152 using the repository's full
  `scripts/check.sh` gate. No physical Scribe test is required for this task. The parallel
  Kindle-release job stopped before cross-compilation because its nested `rustup` setup
  reported an already-installed rustfmt component conflict; that infrastructure failure did
  not occur in the host acceptance gate or Task 24 renderer code.

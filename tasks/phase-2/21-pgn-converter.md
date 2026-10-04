# Task 21 — Build the PGN-to-JSON converter

**Status:** Ready  
**Depends on:** Task 20  
**Primary area:** host tooling only

## Outcome
Add a deterministic host converter. Recommended dependency: `python-chess` for tooling only; never the Kindle runtime.

## Required behavior
- Read starting FEN/SetUp, main line, recursive variations, comments, and NAGs.
- Require stable `PuzzleId` by default; any fallback ID strategy is explicit opt-in.
- Emit UCI + SAN and complete post-move FEN for every node.
- Emit legacy `solution` from the main line.
- Non-main RAV branches default to `sideline`; first-move `[%role alternative]` marks an alternative and is stripped from visible comments.
- Preserve optional description/difficulty/source metadata from tags/options.
- Emit deterministic node IDs/order and deterministic JSON formatting.
- Report encoded size and warn above 256 KiB legacy compatibility.
- Fail malformed/illegal PGN with puzzle context instead of partial output.

## Tests
Checked-in PGN fixtures cover nested variations, promotion, castling, capture, comments, alternative directive, black-to-move FEN, and invalid input. Verify byte-stable output.

## Non-goals
No app/core/render changes and no engine evaluation.

## Suggested agent
Well suited to a capable local model or Codex because it is isolated fixture-driven tooling.

## Suggested commit
`tools: add deterministic PGN puzzle converter`

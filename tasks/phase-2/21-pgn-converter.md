# Task 21 — Build the PGN-to-JSON converter

**Status:** Implemented  
**Depends on:** Task 20  
**Primary area:** host tooling only  
**Validation:** Automated only  
**Physical Scribe required to complete:** No  
**Human interaction required to complete:** No  
**Completion gate:** Converter fixture/unit/integration tests; no device test


## Device/human-testing rule
This task must not request Kindle access. The converter is a host-side authoring tool and its output is fully testable from PGN/JSON fixtures.

## Outcome
Add a deterministic host converter. Recommended dependency: `python-chess` for tooling only; never the Kindle runtime.

## Required behavior
- Read starting FEN/SetUp, main line, recursive variations, comments, and NAGs.
- Require stable `PuzzleId` by default; any fallback ID strategy is explicit opt-in.
- Emit UCI + SAN and complete post-move FEN for every node.
- Emit legacy `solution` from the main line.
- Non-main RAV branches default to `sideline`; first-move `[%role alternative]` marks an alternative and is stripped from visible comments.
- Convert the Task-20 explicit inline move-reference directive into structured `move_ref` spans for descriptions/comments.
- Resolve every inline reference to an existing analysis node/path and fail with context if it is ambiguous or missing.
- Never regex-detect SAN/UCI in ordinary prose; unmarked move-looking text remains a `text` span.
- Emit plain `description`/`comment` projections alongside structured content for compatibility.
- Preserve optional description/difficulty/source metadata from tags/options.
- Emit deterministic node IDs/order and deterministic JSON formatting.
- Report encoded size and warn above 256 KiB legacy compatibility.
- Fail malformed/illegal PGN with puzzle context instead of partial output.

## Tests
Checked-in PGN fixtures cover nested variations, promotion, castling, capture, comments, alternative directive, explicit inline move references, unmarked SAN-looking prose, black-to-move FEN, dangling/ambiguous references, and invalid input. Verify byte-stable output.

## Non-goals
No app/core/render changes and no engine evaluation.

## Suggested agent
Well suited to a capable local model or Codex because it is isolated fixture-driven tooling.

## Suggested commit
`tools: add deterministic PGN puzzle converter`

## Implementation record

Implemented on the `phase-2` branch as the host-only `tools/pgn_converter.py` converter using
python-chess (`chess==1.11.2`). The tool requires `PuzzleId` unless deterministic generated IDs
are explicitly enabled, walks ordered recursive variations, validates legal moves, emits SAN/UCI
and complete post-move FENs, projects the PGN main line to legacy `solution`, preserves NAGs and
metadata, and resolves only Task-20 `[%move_ref ...]` UCI-path directives. Dangling or ambiguous
references and malformed/illegal PGN fail before output is written.

`tests/fixtures/pgn-converter/` and `tests/test_pgn_converter.py` cover the automated acceptance
surface, including black-to-move promotion, capture, castling, nested RAVs, alternative-role
stripping, explicit and plain move-looking prose, metadata options, fallback-ID opt-in, size
warnings, failure context, and byte-stable JSON. `scripts/check.sh` runs the converter tests and
CI installs the pinned host dependency from `requirements-tools.txt`. No Kindle/device checkpoint
is required for this task.

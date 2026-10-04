# Task 22 — Add the core rich-analysis model

**Status:** Implemented  
**Depends on:** Task 20  
**Primary area:** `crates/chess-core`  
**Validation:** Automated only  
**Physical Scribe required to complete:** No  
**Human interaction required to complete:** No  
**Completion gate:** Rust parser/model unit and fixture tests; no device test


## Device/human-testing rule
This task must not request Kindle access. Parsing, validation, lookup, and compatibility semantics are pure core behavior.

## Outcome
Parse optional analysis into a validated platform-neutral tree while preserving legacy v1 behavior.

## Required behavior
- Add analysis tree/node/move/role/NAG and structured text-span types without Kindle/render dependencies.
- Accept legacy puzzles with no `analysis` unchanged.
- Validate every Task-20 invariant with puzzle-specific errors.
- Verify root FEN and exact main-path projection to legacy `solution`.
- Parse all node FEN/UCI at load time and expose efficient node/parent/children lookup.
- Parse `description_content` / node `content`; validate every `move_ref` resolves to an existing non-root node.
- Preserve plain `description` / `comment` fallback behavior when structured content is absent.
- Implement/test the Task-20 phase-two byte cap and retain documented 256 KiB old-build compatibility warning.

## Tests
Begin with all Task-20 valid/invalid fixtures plus regression proving existing phase-one fixtures preserve public semantics.

## Non-goals
No browsing state, rendering, hit testing, PGN parsing, or alternative-branch grading.

## Suggested agent
Codex recommended because this touches the compatibility boundary and public Rust model.

## Suggested commit
`core: parse validated rich analysis trees`


## Implementation record

Implemented on the `phase-2` branch in `crates/chess-core`.

- Added a validated, platform-neutral `AnalysisTree` with typed node indexes, O(1) ID lookup,
  parsed FEN/UCI values, parent/child indexes, roles, NAGs, comments, and structured text spans.
- The parser now validates the complete Task-20 graph, root-FEN, main-path, structured-span, and
  `move_ref` contract with puzzle-specific errors while preserving legacy `solution` grading data.
- Missing structured content is normalized from the existing plain `description` / `comment`
  compatibility fields; legacy analysis-free fixtures retain their existing public fields.
- The phase-two raw-file limit is 8 MiB. `LEGACY_PUZZLE_FILE_WARNING_BYTES` retains the documented
  256 KiB old-build warning threshold for producers and compatibility tooling.
- Rust tests consume the full Task-20 valid/invalid fixture corpus, cover additional frozen
  invariants and lookup/default semantics, regress the phase-one fixture, and exercise both byte
  thresholds. No Kindle/device checkpoint is required for this task.

# Task 22 — Add the core rich-analysis model

**Status:** Ready  
**Depends on:** Task 20  
**Primary area:** `crates/chess-core`

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

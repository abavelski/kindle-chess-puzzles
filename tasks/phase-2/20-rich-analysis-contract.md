# Task 20 — Freeze the rich-analysis contract

**Status:** Ready  
**Depends on:** Phase-one closure  
**Primary area:** docs + fixtures

## Outcome
Turn the phase-two proposal into executable fixtures so later tasks do not re-decide the schema.

## Scope
- Add a smallest valid rich fixture with main, sideline, alternative, comment, NAG, black-to-move, promotion, structured text, and `move_ref` coverage.
- Add invalid fixtures for duplicate ID, missing child, cycle/disconnected node, root-FEN mismatch, invalid UCI, and main-path/legacy-solution mismatch.
- Freeze the structured span contract for puzzle `description_content` and node `content`, including `text` and `move_ref`.
- Freeze one explicit PGN/comment authoring directive for references embedded in prose; it must resolve deterministically to an analysis node/path and must not depend on regex-detecting SAN.
- Add invalid fixtures for dangling/root `move_ref` targets and malformed structured spans.
- Keep an untouched legacy v1 fixture as compatibility control.
- Specify defaults for omitted role/comment/content/NAG/children.
- Measure representative generated book data and record a concrete phase-two byte cap while retaining the old-build 256 KiB warning threshold.

## Non-goals
No parser, converter, UI, or Kindle changes.

## Acceptance
A later agent can implement parser/converter tests from these fixtures/docs with no unresolved schema or inline-reference choices. It is explicit which visible moves are interactive, and legacy fixture semantics remain unchanged.

## Suggested agent
Good local-model task: contract review and deterministic fixture authoring.

## Suggested commit
`docs: freeze rich analysis fixture contract`

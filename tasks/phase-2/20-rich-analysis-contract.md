# Task 20 — Freeze the rich-analysis contract

**Status:** Ready  
**Depends on:** Phase-one closure  
**Primary area:** docs + fixtures

## Outcome
Turn the phase-two proposal into executable fixtures so later tasks do not re-decide the schema.

## Scope
- Add a smallest valid rich fixture with main, sideline, alternative, comment, NAG, black-to-move, and promotion coverage.
- Add invalid fixtures for duplicate ID, missing child, cycle/disconnected node, root-FEN mismatch, invalid UCI, and main-path/legacy-solution mismatch.
- Keep an untouched legacy v1 fixture as compatibility control.
- Specify defaults for omitted role/comment/NAG/children.
- Measure representative generated book data and record a concrete phase-two byte cap while retaining the old-build 256 KiB warning threshold.

## Non-goals
No parser, converter, UI, or Kindle changes.

## Acceptance
A later agent can implement parser tests from these fixtures/docs with no unresolved schema choices. Legacy fixture semantics remain unchanged.

## Suggested agent
Good local-model task: contract review and deterministic fixture authoring.

## Suggested commit
`docs: freeze rich analysis fixture contract`

# Source and fixture provenance

## Reference application

Task 01 reimplements the platform-neutral behavior documented and tested in:

- https://github.com/abavelski/eink-chess-app

The Kindle workspace does not import Cobalt or its Kobo UI/runtime code. The core is structured as a separate crate with explicit FEN, UCI, collection-name, puzzle, and progress modules so a future Kobo adapter can reuse the same implementation.

The reference repository does not currently contain a top-level license file. Treat its source as a behavioral/reference source unless its licensing is clarified; do not assume a redistribution license for unrelated source or assets.

## Puzzle fixtures

`tests/fixtures/puzzles.json` is adapted from the reference repository's Lichess sample collection. The fixture declares:

- source: `https://database.lichess.org/#puzzles`
- license: `CC0-1.0`

`tests/fixtures/promotion-puzzles.json` mirrors the reference repository's small promotion compatibility cases. It is retained only as project test data for matching the reference behavior.

## Sashité assets

No Sashité image assets are added by Task 01. Their source/provenance remains documented in the project plan and will be handled when Task 03 introduces generated renderer assets.

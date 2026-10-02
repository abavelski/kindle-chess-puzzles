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

Task 03 adds the same Sashité Western chess SVG source set used by the reference application:

- upstream project: `https://sashite.dev/assets/chess/`;
- local source of truth: `assets/sashite-western/first/*.svg` and `assets/sashite-western/second/*.svg`;
- piece set: king, queen, rook, bishop, knight, and pawn for both players;
- upstream/reference attribution: public domain, used here under CC0 1.0.

The SVG files are preserved as source assets. `scripts/generate_sashite.py` deterministically converts their supported path/fill data into normalized grayscale vector layers in `crates/chess-render/src/pieces_generated.rs`. Runtime code rasterizes those generated layers into the project-owned Gray8 frame and does not parse SVG on the Kindle.

`scripts/check.sh` runs `python3 scripts/generate_sashite.py --check` so CI fails if the generated Rust output no longer matches the checked-in SVG sources. Additional asset notes live in `assets/sashite-western/README.md`.

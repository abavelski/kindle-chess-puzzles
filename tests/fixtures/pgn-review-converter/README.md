# PGN review converter fixtures

Task 42 coverage for ordinary PGN import: standard metadata without custom tags, Unicode annotations and RAVs, multiple games, custom FEN/SetUp, generated-ID stability across annotation-only edits, explicit ID overrides, duplicate IDs, malformed/illegal input, and deterministic encoding.

Task 48 adds `annotated-unicode.json`, the deterministic pretty-printed output of:

```sh
python3 tools/pgn_review_converter.py tests/fixtures/pgn-review-converter/annotated-unicode.pgn -o tests/fixtures/pgn-review-converter/annotated-unicode.json
```

`chess-render/tests/review_import.rs` runs this converter on the synthetic PGN,
compares exact output bytes, and exercises runtime parsing, stored positions,
rendered targets, navigation, scratch state, snapshots, and damage replay. This
fixture is host acceptance evidence, not a physical Scribe result.

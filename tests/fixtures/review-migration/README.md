# Standalone Review migration fixtures

`legacy-settings.json` deliberately has different puzzle/review board sizes.
`future-state.json` and `interrupted-state.bin` are protected opaque bytes: an attempted save must never replace them. The source game fixture
is `../game-review/valid-standard.json`; the tests capture a stable authored move
and orientation as legacy resume. Puzzle progress deliberately contains opaque
future bytes and must remain byte-identical.

`kindle-app/tests/review_migration.rs` creates isolated old/new directory fixtures
from these inputs and covers fresh, legacy-only, both-present, invalid/future,
interrupted state and traversal keys. Removing disposable new-root state proves
Review uses defaults rather than restoring legacy state.

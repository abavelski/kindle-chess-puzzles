# Portability and application reuse

**Decision (2026-11-08): prioritize product reuse before hardware portability.** The [original proposal](../archive/docs/PORTABILITY_AND_APP_REUSE_ORIGINAL.md) emphasized Kobo first. That order is superseded by [the completed app-split implementation](../archive/tasks/implemented/app-split.md).

| Product | Kindle Scribe gen 1 | Future Kobo |
| --- | --- | --- |
| Puzzles | Implemented; device accepted | Deferred |
| Game Review | Implemented; device accepted | Deferred |

Keep a single Cargo monorepo, one set of chess/analysis and grayscale UI components, separate product state/actions/screens, and thin product/platform compositions. Do not fork the existing binary or copy its renderer just to get two packages.

### Decisions for the first milestone

- **Independent apps** means distinct Rust binary entrypoints and real package identities, not a runtime flag that hides the existing workspace toggle.
- Generic board/FEN/UCI/analysis and e-ink text, pieces, geometry, damage should remain shared; puzzle validation/grading and review navigation/resume should not be shared product state.
- The Puzzles package/root stays compatible with installations already on the Scribe. Review gets its own writable files and tested, non-destructive legacy import/fallback.
- Physical Chessnut is Puzzles-only initially. Respect live-device ownership and the existing manual Send/input contract.
- Each app has a unique source-verified build receipt, executable, stage, KPM manifest/ID, library scriptlet, data/log directory, and package. A global display lock prevents simultaneous framebuffer owners.
- Continue offline PGN conversion; no runtime chess engine or parser.
- Use behavior-preserving, test-first, one-task-at-a-time refactoring. Human hardware acceptance is the final gate.

### Second milestone (deferred)

Only after both Kindle products work independently should a Kobo backend be added. Separate common logical events and display metrics from device input/refresh/lifecycle; probe framebuffer orientation, touch/button mapping, suspend and FBInk build specifics on actual Kobo hardware. Do not add platform-specific conditionals to both product crates.

See [current architecture](ARCHITECTURE.md) for exact files/interfaces and [completed task plan](../archive/tasks/implemented/app-split.md) for dependencies and acceptance.

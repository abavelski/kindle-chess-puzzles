# Phase 3 — Game review mode

**Status:** Implemented — checkpoint 49A passed 2026-10-07
**Implementation branch:** `game-review`; integration into `main` authorized 2026-10-07
**Baseline:** `main` at `fe61ebd`  
**Scope:** this document and Tasks 41–49 define the completed review phase.
**Acceptance:** host gates and the ARMv7/glibc-2.35 release build passed; the user
confirmed the physical Scribe checkpoint. See [the device record](device/ks1-barolo.md#human-checkpoint-49a--passed-2026-10-07).

## Goal

Add a second top-level workspace for reviewing complete chess games — the user's own games or annotated grandmaster games — while preserving the existing puzzle workflow unchanged.

Review mode should feel like the current app rather than a separate application:

- a small workspace icon sits in the header immediately beside **Refresh**;
- tapping it switches between **Puzzles** and **Game Review**;
- the board remains visible and can use the existing STANDARD/SMALL setting;
- the lower panel shows book-style PGN movetext, comments, NAGs, and variations;
- tapping a rendered move shows its exact precomputed position;
- the second control row uses **PREV / NEXT** to step backward/forward through the game's main line;
- a **GAMES** picker reuses the collection-picker interaction style;
- Free Board remains available for exploring from the currently reviewed position.

The Kindle remains an offline deterministic viewer. It does not become a chess engine or a runtime PGN parser.

## Why this fits the existing architecture

Most of the difficult infrastructure already exists.

### Reusable without conceptual change

- `Board`, FEN parsing, UCI parsing, orientation, selection, and permissive free-board movement.
- `AnalysisTree`, including ordered children, SAN/UCI, roles, NAGs, comments, structured `move_ref` spans, and complete post-move FENs.
- Inline PGN-style analysis rendering and its move hit targets.
- Finger/stylus input normalization.
- DPI-aware toolbar placement and modal geometry.
- STANDARD/SMALL board layouts and the larger small-board text region.
- Gray8 rendering, regional damage calculation, FBInk presentation, lifecycle, packaging, and device input.

### Seams that need to be generalized

The reusable pieces are currently surrounded by puzzle-specific APIs:

1. **Parsing:** `parse_analysis` validates that the analysis main path exactly projects to a puzzle `solution`. Game review needs the same tree validation without a graded solution.
2. **Application state:** `AppState` assumes an active puzzle, puzzle progress, a grading cursor, and puzzle navigation.
3. **Analysis rendering:** the analysis panel reads `AppState::active_puzzle()` and puzzle browser state directly rather than a small analysis-view interface.
4. **Layout/hit targets:** the header, FILES button, GOTO row, and actions are puzzle-specific even though their geometry is reusable.
5. **Storage:** `KindleStorage` discovers only `puzzles*.json` under the puzzle directory.

The implementation should generalize those seams narrowly. It should not rewrite the existing puzzle state machine into a new UI framework.

## Architectural direction

Keep one top-level deterministic app state with two workspaces:

```text
AppState
  workspace: Puzzles | Review
  puzzle: existing puzzle state (preserved)
  review: ReviewSession
  shared settings / transient message
```

The exact Rust field split may stay incremental, but the semantic rule is important: switching to review must not rebuild, reset, or reinterpret the puzzle session. Switching back returns to the exact puzzle board, grading cursor, feedback, open/closed analysis state, puzzle index, and progress state that existed before the switch.

A review session is independent:

```text
ReviewSession
  active game
  selected analysis node
  main-line cursor
  analysis page / focus state
  optional free-board scratch board
  flipped / orientation lock
  game picker state
```

No review action emits `ProgressChanged`. Puzzle progress remains puzzle-only.

## Review data contract

Task 41 freezes the authoritative version-1 contract in [`GAME_REVIEW_FORMAT.md`](GAME_REVIEW_FORMAT.md), backed by the executable corpus in `tests/fixtures/game-review/`.

PGN remains an offline authoring/import format. A host tool converts one or more PGN games into deterministic JSON before copying files to the Kindle.

Version-1 root shape:

```json
{
  "version": 1,
  "title": "My games",
  "games": [
    {
      "id": "stable-game-id",
      "fen": "start position FEN",
      "white": "White player",
      "black": "Black player",
      "result": "1-0",
      "event": "Event",
      "site": "Site",
      "date": "2026.10.07",
      "round": "3",
      "source": "optional source",
      "analysis": {
        "version": 1,
        "root": "n0",
        "nodes": []
      }
    }
  ]
}
```

The exact optional/required fields, filename rules, 8 MiB cap, main-line semantics, and validation rules are frozen in `GAME_REVIEW_FORMAT.md`.

The nested `analysis` object should use the existing phase-two tree contract. Review parsing should share the same graph/FEN/UCI/SAN/comment/`move_ref` validation. The puzzle parser additionally keeps its existing “analysis main line must equal `solution`” invariant; review parsing instead requires a non-empty ordered main line but has no `solution` field.

Game IDs are durable identities for review selection and future resume features. Unlike puzzle PGNs, ordinary game PGNs should not require a custom `PuzzleId` tag. The review converter should generate a stable ID by default from canonical game identity/moves, while allowing an explicit override tag.

## Runtime PGN policy

Do not parse PGN or generate legal moves on the Kindle.

The host converter should:

- parse standard PGN headers;
- walk the main line and recursive annotation variations;
- validate moves with the host chess library;
- emit SAN/UCI plus complete post-move FEN for every node;
- preserve comments and NAGs;
- preserve explicit structured move references where supported;
- generate deterministic IDs/order/JSON.

The runtime only loads validated JSON and displays stored positions.

## Game library and selector

Use a separate Kindle directory, proposed as:

```text
/mnt/us/kindle-chess/games/
  games.json
  games-my-games.json
  games-classics.json
```

The platform adapter discovers `games.json` and `games-*.json`. A file may contain multiple games.

The **GAMES** picker presents one flat paginated list of games across all valid discovered files, using stable keys composed from collection filename + game ID. This avoids adding a second “game collection” picker hierarchy.

Suggested display label:

```text
White — Black
Event / date · result
```

Invalid review files should be surfaced similarly to invalid puzzle collection entries, without making valid games unusable.

If no valid game exists, the review icon remains visible but attempting to enter review mode leaves the puzzle workspace unchanged and reports a transient “No review games installed” message.

## Review navigation semantics

The analysis root represents the position before the first move.

- **NEXT** at the root selects the first main-line move.
- **PREV** at the root is disabled.
- **PREV / NEXT** always navigate the authored main line, never “first child of whatever variation is selected.”
- Tapping a main-line move synchronizes the main-line cursor to that ply.
- Tapping a variation previews that exact node but anchors the main-line cursor at the nearest main-line ancestor/branch point. The next PREV/NEXT returns to predictable main-line navigation.
- At the final main-line node, NEXT is disabled.

The selected move should be visibly highlighted in the movetext. Main-line navigation should focus the analysis page containing the newly selected main-line move; manual page browsing remains available when the game does not fit on one page.

## Free Board semantics in review mode

Review Free Board is transient scratch state, not game editing.

- Enabling FREE clones the currently selected review position.
- Board taps mutate only that scratch board using the existing permissive board behavior.
- Selecting another analysis move or using PREV/NEXT updates the selected review position and resets the scratch board to that new position while FREE remains enabled.
- RESET while FREE is enabled restores the scratch board to the currently selected review position.
- Disabling FREE restores the exact selected authored position.
- Free-board changes are never written to the review JSON and never affect puzzle progress.

## UI direction

### Header

Puzzle mode remains visually unchanged except for the new workspace icon immediately after Refresh.

In review mode:

- Refresh stays at the far left.
- The new workspace icon is selected/inverted to indicate Review mode.
- The right-side FILES control becomes **GAMES** and opens the review picker.
- Settings and Close keep their existing locations.
- Header text shows compact game identity/move context instead of puzzle ID/difficulty/“side to move”.

The workspace toggle must be understandable in monochrome without relying on color.

### First control row

Reuse the dynamic toolbar geometry, but show only controls meaningful to review:

- **FREE**
- **RESET** (reset review scratch board to selected authored position)
- **LOCK**
- **FLIP**

Puzzle-only ANALYSIS and NOTE controls are not shown because review movetext/annotations are the primary always-visible content.

### Second control row

Use two large touch-safe controls:

```text
< PREV                         NEXT >
```

These step through the main line. Do not inherit puzzle GOTO semantics in the first implementation. Direct move-number jumping can be a later improvement.

### Bottom panel

The review panel is always the analysis/movetext surface. Reuse the inline PGN flow, comments, NAG symbols, variation parentheses, structured move references, selected move treatment, and overflow page controls.

The SMALL board setting is especially useful here and must preserve its larger text scale/panel behavior.

## Renderer boundary

The existing analysis panel should stop depending on the full puzzle `AppState`. Introduce the smallest read-only input needed for rendering, conceptually:

```text
AnalysisView
  tree
  root/description spans if any
  selected node
  requested/current page
  display scale/layout
```

Both puzzle analysis and review mode can build that view. This preserves one rendering implementation and one hit-target model for annotated movetext.

Do not introduce a general-purpose widget framework.

## Storage and persistence

The initial Phase 3 implementation added review-library discovery/loading without durable review progress. Task 53 now adds a separate versioned `review-resume.json` for the active collection/game and authored move path/FEN, orientation and analysis page. Puzzle `progress.json` remains unchanged; Free Board experiments stay transient.

For the original first implementation:

- puzzle progress remains exactly as-is;
- settings remain shared;
- the active review game and ply are in-memory only;
- review source files are immutable from the app's point of view.

Task 53 implements the separate review-resume document. Do not overload `progress.json`.

## Refresh and damage behavior

Workspace switching, game switching, main-line navigation, variation selection, and free-board movement must continue using Gray8 frames and calculated damage. A workspace switch may legitimately touch most of the screen, but ordinary move navigation should remain regional where possible.

No new waveform or FBInk policy belongs in this phase.

## Task order

1. Task 41 — freeze review JSON contract and fixtures.
2. Task 42 — build deterministic PGN -> review JSON tooling.
3. Task 43 — add the core review collection/model and shared analysis parsing seam.
4. Task 44 — add review workspace/session state, main-line navigation, picker state, and free-board semantics.
5. Task 45 — render the review workspace, header toggle, two control rows, and always-visible movetext.
6. Task 46 — wire game-picker and movetext/control hit testing.
7. Task 47 — add Kindle review-library discovery/loading and binary effect wiring.
8. Task 48 — add import/update workflow docs and end-to-end host validation.
9. Task 49 — physical Scribe validation and phase closure.

Tasks 41–48 are host-testable and should not require Kindle access. Task 49 is the human/device checkpoint.

## Non-goals for the first review phase

- no chess engine or evaluations generated on Kindle;
- no runtime PGN parser;
- no runtime legal-move generator;
- no editing/reordering annotations on Kindle;
- no saving Free Board experiments into the source game;
- no durable review “read progress” yet;
- no opening explorer, database search, engine arrows, clocks, or evaluation graph;
- no change to puzzle grading semantics or puzzle progress schema;
- no direct move-number GOTO until the basic PREV/NEXT flow is proven on-device.

## Definition of done

Game review is ready only when:

- annotated standard PGNs can be converted offline without custom puzzle metadata;
- review JSON reuses the validated analysis tree and precomputed FEN model;
- switching workspaces preserves the exact puzzle session and puzzle progress;
- GAMES selects among discovered games;
- PREV/NEXT traverse the main line deterministically;
- tapping main/variation moves displays the correct stored FEN;
- review Free Board is reversible scratch state;
- STANDARD and SMALL layouts are snapshot-tested;
- finger and stylus taps use the same shared hit path;
- the final Scribe checkpoint confirms readability, controls, refresh behavior, and return to puzzle mode.

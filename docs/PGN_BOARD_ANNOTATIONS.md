# PGN board annotations — deferred feature

Recorded 2026-10-07. Status: proposed, deferred behind higher-priority work.
This document is a feature proposal, not an implemented format contract or a
request to implement it. Existing Tasks 41–49 remain complete.

## Established syntax and support

`%csl` and `%cal` are established PGN comment extensions used in annotated
material, including [Lichess studies](https://lichess.org/study/embed/3hXzTQFU/I0X4CKtN).
Support varies between viewers. Our pinned host dependency, python-chess 1.11.2,
already extracts both with
[`GameNode.arrows()`](https://python-chess.readthedocs.io/en/stable/pgn.html#chess.pgn.GameNode.arrows).

| Comment directive | Meaning |
| --- | --- |
| `[%csl Ga7,Gc4]` | Mark squares a7 and c4 in green. |
| `[%cal Ga4c4,Ga4a7]` | Draw arrows from a4 to c4 and a4 to a7 in green. |

Color prefixes are `G` (green), `R` (red), `Y` (yellow), and `B` (blue).
The [python-chess parser source](https://python-chess.readthedocs.io/en/stable/_modules/chess/pgn.html)
documents the accepted coordinate/color syntax. Viewers may use rings or fills
for square marks. An arrow describes authored illustration, not a legal move.

The real Fischer–Sherwin PGN in `data/game1.pgn` has three marked positions:
the 21.Qa4 variation has rings on a7/c4 and arrows a4→c4/a4→a7; other variations
include Qe2 with e2→h5 and Nxd2 with c6/e7 marks and f6→e7/g2→c6 arrows.

## Temporary import behavior

Until support exists, `tools/clean_pgn.py` removes complete `%csl`/`%cal`
directives from comments so raw markup does not appear in the reading panel.
It also unwraps `%pre` while retaining prose. Always retain the original PGN:
the cleaned copy intentionally loses the board marks. The converter itself
continues preserving comments; it does not interpret these board directives.

## Proposed implementation sequence

Create separately scoped numbered improvement tasks when this work is prioritized.
Each behavioral task begins with a failing automated test.

1. **Contract and offline conversion.** Specify optional square/arrow data on
   shared analysis nodes, including root nodes. Preserve source colors, coordinate
   order, and stable game IDs. Existing collections without annotations retain
   their semantics. Extract directives on the host into deterministic JSON;
   remove only successfully parsed directives from displayed comment/content
   text, preserve surrounding prose and explicit move references, and report
   malformed directives. Separate pre-variation comments from post-move comments
   when assigning position marks: the current converter joins these prose fields,
   which must not attach pre-move illustrations to the wrong position.
2. **Core model and visibility.** Validate coordinates, colors, and finite annotation
   limits in the shared parser. Expose immutable annotations for the selected
   authored position. Show them in review and puzzle analysis previews; suppress
   them during graded puzzle attempts to avoid hints and during Free Board scratch
   exploration to avoid misleading marks. Navigating away removes old marks.
3. **Monochrome renderer.** Render square rings and straight arrows in Gray8.
   Prototype black strokes with white outlines for contrast over light/dark squares
   and pieces, with minimal obstruction of piece silhouettes. Preserve source color
   metadata but initially use one high-contrast style for all colors; differentiated
   monochrome patterns can follow if real material requires them. Scale geometry
   for STANDARD/SMALL and transform coordinates under FLIP/LOCK. Reuse calculated
   regional damage; no waveform changes or engine/runtime PGN parsing.
4. **Host and physical validation.** Cover root/main/variation marks, malformed
   directives, comment cleanup, unchanged move references, stable identity,
   orientation, crossing/overlapping arrows, occupied endpoints, visibility rules,
   immutable sources and unchanged puzzle progress. Add STANDARD/SMALL snapshots
   and verify damage replay reconstructs every pixel, including mark removal.
   Reconvert the original Fischer–Sherwin PGN with the new converter and verify
   readability, navigation and ghosting on the physical Scribe.

The first release would provide automatic authored marks, square rings and
straight arrows. Drawing/editing annotations, color-specific patterns, and new
controls are outside that initial scope. Piece readability on monochrome e-ink
is the main visual design question to resolve through snapshots and device review.

# Importing and updating review games

Game Review accepts precomputed JSON generated on your computer. PGN parsing and
legal move validation run only in the host converter; the Kindle displays stored
positions. Task 48 verifies this workflow on the host. Task 49 physical Scribe acceptance
passed on 2026-10-07.

## Convert a PGN

From the repository root, prepare the host dependency once:

```sh
python3 -m venv .venv
. .venv/bin/activate
python3 -m pip install -r requirements-tools.txt
```

Convert one or more ordinary or annotated PGN games with one command:

```sh
python3 tools/pgn_review_converter.py my-games.pgn -o games-my-games.json --title "My games"
```

No `PuzzleId` tag is needed. The converter preserves player/event/date/result
headers, comments, recursive variations, NAGs, and Unicode. Each move receives
SAN, UCI, and a complete six-field post-move FEN. Custom `SetUp`/`FEN` starting
positions are supported. The same input/options produce identical UTF-8 JSON
bytes. `--compact` reduces output size; the hard limit is **8 MiB per file**,
reported after conversion. Split larger PGNs into separate named collections.
Use `python3 tools/pgn_review_converter.py --help` for metadata options.

Only explicitly authored references in comments become prose links:

```text
Compare [%move_ref d2d4/d7d5/c2c4|2.c4] with plain Nc3.
```

The path follows UCI moves from the game root, including a promotion suffix when
needed. Omit `|label` to display the target SAN. A missing/ambiguous path fails
conversion. Plain SAN or UCI text stays non-interactive. See
[the shared span contract](PUZZLE_FORMAT.md#pgncomment-authoring-directive-for-inline-move-references).

## Copy and use

Exit the app before copying or replacing game files. Create the directory if it
does not exist, then copy your JSON using USB storage or your established SSH
transport into:

```text
/mnt/us/kindle-chess/games/games.json
/mnt/us/kindle-chess/games/games-my-games.json
/mnt/us/kindle-chess/games/games-classics.json
```

Only `games.json` and `games-<nonempty>.json` are discovered. The directory is
configurable through `KINDLE_CHESS_GAME_DIR`. Relaunch to reload the library.
PGN files themselves are not loaded by the app. Invalid JSON files appear as
non-selectable diagnostic rows while valid games remain available.

Tap the workspace icon immediately beside Refresh to enter Game Review. **GAMES**
opens one flat, paginated list across all discovered files. **PREV/NEXT** traverse
the authored main line; tapping a bold move or explicit prose reference displays
its stored position. A variation anchors navigation at its nearest main-line
ancestor: NEXT selects the following main-line ply, PREV the preceding one.
At the root PREV is disabled, and at the final main-line move NEXT is disabled.

**FREE** creates a scratch board from the selected position. **RESET** restores
that position; choosing another move or PREV/NEXT resets scratch to the newly
selected position while FREE stays enabled. Turning FREE off restores the
selected authored position. **LOCK/FLIP** control review orientation. Settings
are shared, including STANDARD/SMALL board size. Switching back to Puzzles
restores the in-progress puzzle session and leaves its progress untouched.
With no installed valid games, the toggle leaves Puzzles active and reports
“No review games installed”. Review game/ply selection is in-memory only.

## Update and recover

Keep the original PGN and a backup of the generated JSON. Rerun the same converter
command targeting the same filename after editing annotations. The converter
validates every game, legal move, explicit reference, resulting ID uniqueness,
and output size **before** writing. It writes a temporary file beside the output,
flushes/syncs it, then replaces the destination atomically. Conversion failures or
failures before replacement preserve the old bytes; write failures return a
nonzero status with an error message. The destination's parent directory must
already exist. This replaces the whole collection; it does not append or merge.

For example, before an update:

```sh
cp games-my-games.json games-my-games.json.bak
python3 tools/pgn_review_converter.py my-games.pgn -o games-my-games.json --title "My games"
```

Copy the successfully generated file to the Kindle while the app is closed.
For rollback, restore the backed-up JSON to the same filename and relaunch.
Do not redirect stdout over an existing file (`> games-my-games.json`): the shell
would truncate it before conversion. Use `-o` for safe replacement. Atomic host
output replacement does not make a USB/network copy atomic; retain the backup
until the copied file loads successfully.

Generated game IDs hash the seven identity headers (Event, Site, Date, Round,
White, Black, Result), starting FEN, and main-line UCI moves. Changes limited to
comments, NAGs, or variations retain the ID. Editing those headers, the starting
position, or main line changes it. To keep an explicitly managed identity, add
`[GameId "my-stable-id"]` to the PGN, use a custom `--id-tag`, or use `--game-id`
for a single-game input. Two games with the same resulting ID fail clearly,
including identical games imported twice; give distinct games distinct explicit
IDs or remove duplicate records. Runtime keys combine the collection filename
and game ID, so keep filenames stable too. No durable review resume exists yet.

The converter never reads or writes `progress.json`, and runtime review actions
never modify game sources or puzzle grading/progress. Backups ending in `.bak`
are not discovered as collections.

## Host validation

With the virtual environment active:

```sh
scripts/check.sh
scripts/build-kindle.sh
```

The Rust `review_import` integration test invokes the real converter on the
annotated Unicode PGN fixture, compares bytes with its checked-in JSON, parses
that output, traverses the entire main line both ways, and taps every rendered
move/reference. It checks exact stored FEN previews, variation return semantics,
FREE/reset/exit, immutable source data, restoration of an in-progress puzzle,
and identical serialized puzzle progress. STANDARD/SMALL snapshots are visually
reviewed, and replaying regional damage must reconstruct every changed pixel.
Inspectable snapshot frames are written under `target/review-import-*.pgm`.
Python regression tests cover successful annotation updates and rejected
illegal/colliding updates preserving existing bytes. The pinned Kindle build
checks ARM ABI/glibc compatibility; it is not physical acceptance.

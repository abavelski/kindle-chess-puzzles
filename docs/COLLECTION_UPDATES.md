# Safe puzzle collection updates

Phase two treats puzzle collections as replaceable source data and progress as a separate durable store. Regenerating `puzzles*.json` must preserve the identities that progress refers to, surface risky changes before replacement, and replace only the collection file.

## Identity and versioning roles

These values have different jobs and must not be used interchangeably:

| Value | Role | Update rule |
| --- | --- | --- |
| root `version` | Collection schema major. Phase two remains schema version `1`. | Change only for an incompatible collection-format revision. |
| `analysis.version` | Rich-analysis extension schema. Phase two uses `1`. | Change only for an incompatible analysis-extension revision. |
| root `revision` | Optional human/content revision for a release/import. | Bump when useful for release tracking; it is not a progress key. |
| puzzle `id` / PGN `PuzzleId` | Durable exercise identity referenced by progress. | Keep stable across reorder, comment, NAG, source, and side-line edits to the same exercise. |
| collection filename | Durable per-collection progress key (`puzzles.json` or `puzzles-<name>.json`). | Keep stable across normal regeneration. |
| Git history | Authoritative audit trail and rollback mechanism. | Commit author sources and generated/update-policy changes normally. |

Assign a **new puzzle ID** when the starting position changes or the exercise meaning changes enough that old solved state should not carry forward. Phase two does not automatically invalidate solved state and does not migrate progress between IDs or filenames.

## Converter safety contract

`tools/pgn_converter.py` is the supported PGN-to-JSON regeneration path. When `-o/--output` names an existing file, the converter reads that collection first and prints a deterministic comparison **before** replacement:

- added IDs;
- removed IDs;
- duplicate IDs;
- IDs whose starting `fen` changed while the ID stayed the same.

Reorder-only and prose/analysis-only edits do not appear as identity changes. Duplicate IDs block replacement. A FEN change is highlighted but does not silently rewrite the ID; the author must decide whether the exercise is truly the same. In normal authoring, a meaningful FEN/exercise change should be accompanied by a new `PuzzleId`.

Successful file output uses a same-directory temporary file, flush + file sync, and atomic rename/replace. Failed conversion, malformed existing JSON, or duplicate-ID validation leaves the previous output in place. Standard-output mode (`-o -`) does not perform replacement comparison because there is no target collection to replace.

The converter never opens or writes the Kindle progress store. Collection JSON contains puzzle/source metadata only; progress remains in the separate platform progress file described in `PUZZLE_FORMAT.md`.

## Size thresholds

Every conversion reports the encoded UTF-8 byte size and both compatibility thresholds:

- **legacy / phase-one warning threshold:** 256 KiB = 262,144 bytes. Older phase-one readers can reject files above this size;
- **phase-two runtime cap:** 8 MiB = 8,388,608 bytes, measured on the raw UTF-8 collection before parsing.

The converter warns above 256 KiB and warns again above 8 MiB. Do not raise the runtime cap casually; split an oversized book into multiple stable collection files.

## Cookbook

### Initial import

Author one PGN game per puzzle with a stable `PuzzleId`, then create the collection:

```sh
python3 tools/pgn_converter.py books/endgames.pgn \
  -o puzzles-endgames.json \
  --title "Endgames" \
  --revision 1
```

Review the size report, inspect the generated JSON, and commit the author source plus any intended generated artifact according to the repository's release practice.

### Regenerate an existing collection

Use the **same output filename** and retain each `PuzzleId` for the same exercise:

```sh
python3 tools/pgn_converter.py books/endgames.pgn \
  -o puzzles-endgames.json \
  --title "Endgames" \
  --revision 2
```

Before the old file is replaced, review the comparison. For a comment/reorder/side-line-only release, `added IDs`, `removed IDs`, and `FEN-changed IDs` should normally all be `none`. If a FEN-changed ID is intentional because the exercise has materially changed, change its PGN `PuzzleId` and regenerate.

### Add or remove puzzles

Add or remove PGN games, then regenerate to the same filename. Confirm that the reported added/removed IDs exactly match the intended edit. Removed IDs remain in historical Git commits; the converter does not edit progress to delete their solved records.

### Split an oversized collection

If output approaches or exceeds 8 MiB, split the author source into multiple collections and give each output a stable name, for example:

```text
puzzles-endgames-a.json
puzzles-endgames-b.json
```

Keep those filenames stable on later regenerations. Progress is keyed first by collection filename, so splitting or renaming an already-deployed collection does **not** migrate per-file current/solved state. Plan such a split as an explicit release change; there is no automatic progress migration service in phase two.

### Roll back with Git

Git is the historical source of truth. Restore the prior authoring revision (or the prior generated collection when that is what the repository tracks), then regenerate/restore to the **same collection filename**. Example author-source rollback:

```sh
git restore --source <good-commit> -- books/endgames.pgn
python3 tools/pgn_converter.py books/endgames.pgn \
  -o puzzles-endgames.json \
  --title "Endgames" \
  --revision <restored-revision>
```

Review the comparison before accepting the replacement. Do not roll back or repair a collection by deleting `progress.json`; progress is intentionally independent data.

## Release checklist

Before committing or copying a regenerated collection, verify that the output filename is unchanged for a normal update, expected exercises kept their IDs, any FEN-changed ID was reviewed, there are no duplicates, the byte-size report is acceptable, and the progress file/hash was not touched. Then commit the update so Git records the exact historical version that was shipped.

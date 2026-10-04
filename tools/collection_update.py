"""Safe comparison and atomic replacement helpers for puzzle collections."""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
import tempfile
from typing import Any


PHASE_TWO_MAX_BYTES = 8 * 1024 * 1024


class CollectionUpdateError(ValueError):
    """An existing/generated collection cannot be compared or safely replaced."""


@dataclass(frozen=True)
class CollectionComparison:
    added_ids: tuple[str, ...]
    removed_ids: tuple[str, ...]
    duplicate_existing_ids: tuple[str, ...]
    duplicate_new_ids: tuple[str, ...]
    fen_changed_ids: tuple[str, ...]

    @property
    def duplicate_ids(self) -> tuple[str, ...]:
        return tuple(sorted(set(self.duplicate_existing_ids) | set(self.duplicate_new_ids)))


def _index_puzzles(collection: dict[str, Any], label: str) -> tuple[dict[str, str], tuple[str, ...]]:
    if not isinstance(collection, dict):
        raise CollectionUpdateError(f"{label}: collection root must be an object")
    puzzles = collection.get("puzzles")
    if not isinstance(puzzles, list):
        raise CollectionUpdateError(f"{label}: puzzles must be an array")

    by_id: dict[str, str] = {}
    duplicates: set[str] = set()
    for index, puzzle in enumerate(puzzles):
        context = f"{label}: puzzle #{index + 1}"
        if not isinstance(puzzle, dict):
            raise CollectionUpdateError(f"{context} must be an object")
        puzzle_id = puzzle.get("id")
        if not isinstance(puzzle_id, str) or not puzzle_id.strip():
            raise CollectionUpdateError(f"{context} id must be a non-empty string")
        fen = puzzle.get("fen")
        if not isinstance(fen, str) or not fen.strip():
            raise CollectionUpdateError(f"{context} ({puzzle_id}) fen must be a non-empty string")
        if puzzle_id in by_id:
            duplicates.add(puzzle_id)
        else:
            by_id[puzzle_id] = fen
    return by_id, tuple(sorted(duplicates))


def compare_collections(previous: dict[str, Any], new: dict[str, Any]) -> CollectionComparison:
    """Compare durable puzzle identity and starting positions, ignoring order/content edits."""

    previous_by_id, previous_duplicates = _index_puzzles(previous, "existing collection")
    new_by_id, new_duplicates = _index_puzzles(new, "generated collection")
    previous_ids = set(previous_by_id)
    new_ids = set(new_by_id)
    shared = previous_ids & new_ids
    return CollectionComparison(
        added_ids=tuple(sorted(new_ids - previous_ids)),
        removed_ids=tuple(sorted(previous_ids - new_ids)),
        duplicate_existing_ids=previous_duplicates,
        duplicate_new_ids=new_duplicates,
        fen_changed_ids=tuple(
            sorted(puzzle_id for puzzle_id in shared if previous_by_id[puzzle_id] != new_by_id[puzzle_id])
        ),
    )


def comparison_messages(comparison: CollectionComparison) -> list[str]:
    def render(ids: tuple[str, ...]) -> str:
        return ", ".join(ids) if ids else "none"

    return [
        "collection update comparison:",
        f"  added IDs: {render(comparison.added_ids)}",
        f"  removed IDs: {render(comparison.removed_ids)}",
        f"  duplicate IDs: {render(comparison.duplicate_ids)}",
        f"  FEN-changed IDs: {render(comparison.fen_changed_ids)}",
    ]


def ensure_no_duplicate_ids(comparison: CollectionComparison) -> None:
    if comparison.duplicate_ids:
        raise CollectionUpdateError(
            "duplicate puzzle IDs block replacement: " + ", ".join(comparison.duplicate_ids)
        )


def load_collection_file(path: Path) -> dict[str, Any]:
    try:
        decoded = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise CollectionUpdateError(f"cannot read existing collection {path}: {exc}") from exc
    if not isinstance(decoded, dict):
        raise CollectionUpdateError(f"existing collection {path}: root must be an object")
    return decoded


def atomic_write_bytes(path: Path, data: bytes) -> None:
    """Replace one collection path atomically without reading or writing any progress file."""

    parent = path.parent
    try:
        mode = path.stat().st_mode & 0o777
    except FileNotFoundError:
        mode = 0o644
    except OSError as exc:
        raise CollectionUpdateError(f"cannot inspect collection {path}: {exc}") from exc

    fd: int | None = None
    temp_path: Path | None = None
    try:
        fd, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=parent)
        temp_path = Path(temp_name)
        with os.fdopen(fd, "wb") as handle:
            fd = None
            os.fchmod(handle.fileno(), mode)
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_path, path)
        temp_path = None

        # Directory sync is best-effort: the atomic rename has already succeeded,
        # and some host filesystems do not support fsync on directory descriptors.
        try:
            directory_fd = os.open(parent, os.O_RDONLY)
        except OSError:
            directory_fd = None
        if directory_fd is not None:
            try:
                try:
                    os.fsync(directory_fd)
                except OSError:
                    pass
            finally:
                os.close(directory_fd)
    except OSError as exc:
        raise CollectionUpdateError(f"cannot atomically write collection {path}: {exc}") from exc
    finally:
        if fd is not None:
            os.close(fd)
        if temp_path is not None:
            try:
                temp_path.unlink()
            except FileNotFoundError:
                pass

#!/usr/bin/env python3
"""One-time host-driven Review migration. No Python is required on the Kindle."""
from __future__ import annotations

import argparse
import json
from pathlib import Path, PurePosixPath
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def is_game_filename(name):
    return name == 'games.json' or (
        name.startswith('games-') and name.endswith('.json')
        and len(name) > len('games-.json') and '\\' not in name)


class StateValidator:
    """Use the application's exact parsers without reimplementing its schemas."""

    def __init__(self):
        result = subprocess.run(
            ['cargo', 'build', '--locked', '-p', 'chess-core', '--example',
             'validate_migration_state', '--message-format=json'],
            cwd=ROOT, check=True, capture_output=True, text=True)
        executables = [item['executable'] for line in result.stdout.splitlines()
                       if (item := json.loads(line)).get('reason') == 'compiler-artifact'
                       and item.get('executable')]
        self.executable = executables[-1]

    def validate(self, kind, data):
        result = subprocess.run([self.executable, kind], input=data, capture_output=True)
        return None if result.returncode == 0 else result.stderr.decode().strip()


class ShellFiles:
    """Small file operations shared by SSH and local shell tests."""

    def __init__(self, execute):
        self.execute = execute

    def kind(self, path):
        p = shlex.quote(path)
        return self.execute(
            f'if [ -L {p} ]; then echo other; elif [ -f {p} ]; then echo file; '
            f'elif [ -d {p} ]; then echo directory; elif [ -e {p} ]; then echo other; '
            'else echo missing; fi').decode().strip()

    def read(self, path):
        if self.kind(path) != 'file':
            raise RuntimeError(f'not a regular file: {path}')
        return self.execute(f'cat {shlex.quote(path)}')

    def games(self, root):
        directory = str(PurePosixPath(root) / 'games')
        kind = self.kind(directory)
        if kind == 'missing':
            return []
        if kind != 'directory':
            raise RuntimeError(f'not a directory: {directory}')
        # Only the flat JSON library discovered by the application; leave other files alone.
        raw = self.execute(f'find {shlex.quote(directory)} -maxdepth 1 -name "*.json" -print0')
        paths = [p.decode('utf-8') for p in raw.split(b'\0') if p]
        return sorted(path for path in paths if is_game_filename(PurePosixPath(path).name))

    def copy_absent(self, source, target):
        data = self.read(source)
        parent = shlex.quote(str(PurePosixPath(target).parent))
        # noclobber refuses an existing destination; a failed write leaves source intact.
        self.execute(f'mkdir -p {parent} && (set -C; cat > {shlex.quote(target)})', data)
        if self.read(target) != data:
            raise RuntimeError(f'copy verification failed: {target}')

    def remove_identical(self, source, target):
        self.execute(f'cmp -s {shlex.quote(source)} {shlex.quote(target)} '
                     f'&& rm {shlex.quote(source)}')

    def remove_superseded(self, source, expected):
        if self.read(source) != expected:
            raise RuntimeError(f'source changed during migration: {source}')
        self.execute(f'rm {shlex.quote(source)}')


class Migrator:
    def __init__(self, files, validator, report=print):
        self.files = files
        self.validator = validator
        self.report = report
        self.issues = 0

    def attention(self, message):
        self.issues += 1
        self.report(f'Attention: {message}')

    def state(self, path, kind):
        status = self.files.kind(path)
        if status == 'missing':
            return None
        if status != 'file':
            self.attention(f'{path}: not a regular file; left untouched')
            return False
        data = self.files.read(path)
        error = self.validator.validate(kind, data)
        if error:
            self.attention(f'{path}: {error}; left untouched')
            return False
        return data

    def run(self, puzzles, review, check_closed):
        check_closed()
        for root in (puzzles, review):
            for path in (root, f'{root}/state', f'{root}/games'):
                if self.files.kind(path) not in ('missing', 'directory'):
                    raise RuntimeError(f'not a plain directory: {path}')
        moved = present = 0
        for source in self.files.games(puzzles):
            target = str(PurePosixPath(review) / 'games' / PurePosixPath(source).name)
            if self.files.kind(source) != 'file':
                self.attention(f'{source}: not a regular file; left untouched')
                continue
            kind = self.files.kind(target)
            if kind == 'missing':
                self.files.copy_absent(source, target)
                self.files.remove_identical(source, target)
                moved += 1
                self.report(f'Moved: {source} -> {target}')
            elif kind == 'file' and self.files.read(source) == self.files.read(target):
                self.files.remove_identical(source, target)
                present += 1
                self.report(f'Already present: {target}; removed identical legacy copy')
            else:
                self.attention(f'conflict: {source} and {target}; neither overwritten')
        for kind, name in [('settings', 'settings.json'), ('resume', 'review-resume.json')]:
            source = f'{puzzles}/state/{name}'
            target = f'{review}/state/{name}'
            owned = self.state(target, kind)
            old = self.state(source, kind)
            if owned is False:
                continue
            if owned is not None:
                self.report(f'Already present: {target}; kept existing Review state')
                if kind == 'resume' and old is not None and old is not False:
                    self.files.remove_superseded(source, old)
                    self.report(f'Removed superseded legacy resume: {source}')
                continue
            if old is None:
                self.report(f'Absent: {source}; Review will use defaults')
            elif old is not False:
                self.files.copy_absent(source, target)
                if kind == 'resume':
                    self.files.remove_identical(source, target)
                self.report(f'Copied {kind}: {source} -> {target}')
        self.report(f'Game files: {moved} moved, {present} already present; '
                    f'{len(self.files.games(review))} in Review, '
                    f'{len(self.files.games(puzzles))} remaining in Puzzles; '
                    f'{self.issues} requiring attention')
        return 1 if self.issues else 0


def require_closed(execute):
    # Existing supervisor ownership plus direct executables. No new lock or process killing.
    execute('''command -v pidof >/dev/null 2>&1 || { echo 'Cannot check active apps: pidof unavailable' >&2; exit 1; }
for lock in /tmp/kindle-chess.lock /tmp/kindle-game-review.lock /tmp/kindle-chess-display.lock; do
    if [ -e "$lock" ]; then echo "Apps must be closed: $lock exists" >&2; exit 1; fi
done
for app in kindle-chess kindle-game-review kindle-puzzles; do
    if pidof "$app" >/dev/null 2>&1; then echo "Apps must be closed: $app active" >&2; exit 1; fi
done''')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', required=True, help='SSH destination, for example root@DEVICE_IP')
    parser.add_argument('--port', type=int, default=2222)
    args = parser.parse_args()
    if args.host.startswith('-') or not 1 <= args.port <= 65535:
        parser.error('invalid SSH host or port')
    try:
        validator = StateValidator()

        def execute(command, data=None):
            result = subprocess.run(['ssh', '-p', str(args.port), args.host, command],
                                    input=data, capture_output=True, check=True)
            return result.stdout

        return Migrator(ShellFiles(execute), validator).run(
            '/mnt/us/kindle-chess', '/mnt/us/kindle-game-review',
            check_closed=lambda: require_closed(execute))
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        detail = error.stderr.decode(errors='replace').strip() if isinstance(
            error, subprocess.CalledProcessError) and error.stderr else str(error)
        print(f'Migration stopped: {detail}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())

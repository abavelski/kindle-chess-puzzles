import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from tools.migrate_app_data import Migrator, ShellFiles, StateValidator, require_closed

ROOT = Path(__file__).resolve().parents[1]


class MigrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.validator = StateValidator()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.puzzles = self.root / 'kindle-chess'
        self.review = self.root / 'kindle-game-review'
        (self.puzzles / 'games').mkdir(parents=True)
        (self.puzzles / 'state').mkdir()
        self.game = (ROOT / 'tests/fixtures/game-review/valid-standard.json').read_bytes()
        (self.puzzles / 'games' / "games-book 'one'.json").write_bytes(self.game)
        self.settings = (ROOT / 'tests/fixtures/review-migration/legacy-settings.json').read_bytes()
        (self.puzzles / 'state/settings.json').write_bytes(self.settings)
        self.resume = json.dumps({
            'version': 1, 'collection_id': "games-book 'one'.json", 'game_id': 'sample',
            'move_path': ['e2e4'],
            'fen': 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1',
            'flipped': True, 'orientation_locked': False, 'analysis_page': 2,
            'focus_selected': True,
        }).encode()
        (self.puzzles / 'state/review-resume.json').write_bytes(self.resume)
        for name in ['puzzles/book.json', 'state/progress.json', 'runtime/binary',
                     'chessnut/config.json', 'games/readme.txt', 'games/notes.json']:
            path = self.puzzles / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'unrelated opaque data')
        self.preserved = {self.puzzles / name: (self.puzzles / name).read_bytes() for name in
                          ['puzzles/book.json', 'state/progress.json', 'state/settings.json',
                           'runtime/binary', 'chessnut/config.json', 'games/readme.txt', 'games/notes.json']}
        self.messages = []
        # Same shell transport/commands as SSH, executed against temporary local roots.
        self.files = ShellFiles(lambda command, data=None: subprocess.run(
            ['sh', '-c', command], input=data, check=True, capture_output=True).stdout)

    def migrate(self):
        return Migrator(self.files, self.validator, self.messages.append).run(
            str(self.puzzles), str(self.review), check_closed=lambda: None)

    def assert_preserved(self):
        for path, contents in self.preserved.items():
            self.assertEqual(path.read_bytes(), contents)

    def test_ordinary_migration_and_rerun_preserve_unrelated_puzzles(self):
        self.assertEqual(self.migrate(), 0)
        self.assertEqual((self.review / "games/games-book 'one'.json").read_bytes(), self.game)
        self.assertFalse((self.puzzles / "games/games-book 'one'.json").exists())
        self.assertEqual((self.review / 'state/settings.json').read_bytes(), self.settings)
        self.assertEqual((self.review / 'state/review-resume.json').read_bytes(), self.resume)
        self.assertFalse((self.puzzles / 'state/review-resume.json').exists())
        self.assertEqual(self.migrate(), 0)
        self.assert_preserved()

    def test_existing_newer_review_state_and_identical_games_win(self):
        (self.review / 'state').mkdir(parents=True)
        (self.review / 'games').mkdir()
        newer = self.resume.replace(b'e2e4', b'd2d4')
        settings = b'{"version":1,"game_review":{"small_board":false}}'
        (self.review / 'state/settings.json').write_bytes(settings)
        (self.review / 'state/review-resume.json').write_bytes(newer)
        (self.review / "games/games-book 'one'.json").write_bytes(self.game)
        self.assertEqual(self.migrate(), 0)
        self.assertEqual((self.review / 'state/review-resume.json').read_bytes(), newer)
        self.assertEqual((self.review / 'state/settings.json').read_bytes(), settings)
        self.assertFalse((self.puzzles / 'state/review-resume.json').exists())
        self.assertTrue(any('already present' in m for m in self.messages))
        self.assert_preserved()

    def test_conflicting_games_and_invalid_state_are_preserved_and_reported(self):
        (self.review / 'games').mkdir(parents=True)
        (self.review / 'state').mkdir()
        path = self.review / "games/games-book 'one'.json"
        path.write_bytes(b'different')
        invalid = b'{"version":99}'
        (self.review / 'state/review-resume.json').write_bytes(invalid)
        (self.puzzles / 'state/settings.json').write_bytes(b'{"version":1,"small_board":0}')
        self.preserved[self.puzzles / 'state/settings.json'] = b'{"version":1,"small_board":0}'
        self.assertEqual(self.migrate(), 1)
        self.assertEqual(path.read_bytes(), b'different')
        self.assertEqual((self.puzzles / "games/games-book 'one'.json").read_bytes(), self.game)
        self.assertEqual((self.review / 'state/review-resume.json').read_bytes(), invalid)
        self.assertEqual((self.puzzles / 'state/review-resume.json').read_bytes(), self.resume)
        self.assertFalse((self.review / 'state/settings.json').exists())
        self.assertTrue(any('attention' in m.lower() for m in self.messages))
        self.assert_preserved()

    def test_active_app_refuses_before_writing(self):
        bin_dir = self.root / 'bin'
        bin_dir.mkdir()
        pidof = bin_dir / 'pidof'
        pidof.write_text('#!/bin/sh\nexit 0\n')
        pidof.chmod(0o755)

        def active():
            def execute(command):
                result = subprocess.run(['sh', '-c', command], check=False, capture_output=True,
                                        env={**os.environ, 'PATH': f'{bin_dir}:{os.environ["PATH"]}'})
                if result.returncode:
                    raise RuntimeError(result.stderr.decode())
                return result.stdout
            require_closed(execute)
        with self.assertRaisesRegex(RuntimeError, 'closed'):
            Migrator(self.files, self.validator, self.messages.append).run(
                str(self.puzzles), str(self.review), check_closed=active)
        self.assertFalse(self.review.exists())
        self.assertTrue((self.puzzles / 'state/review-resume.json').exists())
        self.assert_preserved()


if __name__ == '__main__':
    unittest.main()

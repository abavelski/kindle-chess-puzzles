"""Play is an independently installable product without libraries or board assets."""
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from tests.test_packaging import workflow, elf
from tests.test_release_matrix import verifier

ROOT = Path(__file__).resolve().parents[1]

class PlayPackageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.base = Path(cls.temporary.name)
        cls.w = workflow()
        product = cls.w.product('play')
        binary = cls.base / product['binary']
        binary.write_bytes(elf('play'))
        receipt = {'app': 'play', **{k: product[k] for k in ['binary', 'package_id', 'runtime_root']},
                   'binary_sha256': cls.w.sha256(binary), 'source_sha256': cls.w.source_digest('play'),
                   'checks': 'passed', 'target': cls.w.TARGET, 'rust': '1.85.1',
                   'zig': cls.w.ZIG_VERSION, 'glibc': '2.35', 'fbink': cls.w.FBINK}
        (cls.base / 'build.json').write_text(json.dumps(receipt))
        cls.w.stage(binary, cls.base / 'build.json', cls.base / 'stage', app='play')
        cls.w.archive(cls.base / 'stage', cls.base / 'kindle-chess-play.tar')
        cls.w.kpkg(cls.base / 'stage', cls.base / 'kindle_chess_play_0.1.0_kindlehf.kpkg')

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def test_artifacts_identities_and_launch_are_play_only(self):
        self.assertEqual(verifier().verify_artifacts('play', self.base)['app'], 'play')
        runtime = self.base / 'stage/runtime'
        launch = (runtime / 'launch.sh').read_text()
        for fragment in ['KINDLE_CHESS_PLAY_ROOT', 'KINDLE_CHESS_PLAY_LOCK', '/tmp/kindle-chess-play.lock']:
            self.assertIn(fragment, launch)
        for forbidden in ['KINDLE_CHESS_PUZZLE_DIR', 'KINDLE_GAME_REVIEW', 'KINDLE_CHESS_PROGRESS_FILE']:
            self.assertNotIn(forbidden, launch)
        self.assertEqual((runtime / 'kindle_launch.sh').read_bytes(), (ROOT / 'scripts/kindle_launch.sh').read_bytes())
        for name in ['games', 'puzzles', 'state', 'config.json', 'kindlebt-worker', 'kindle-chess', 'kindle-game-review']:
            self.assertFalse((runtime / name).exists(), name)
        for app in ['puzzles', 'review']:
            with self.assertRaises(ValueError):
                self.w.verify_stage(self.base / 'stage', app=app)
        self.assertEqual(len({self.w.output_dir(app) for app in self.w.PRODUCTS}), 3)

    def test_play_rejects_foreign_assets_even_with_valid_checksums(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / 'play'
            shutil.copytree(self.base, directory)
            stage = directory / 'stage'
            for name in ['games.json', 'puzzles.json', 'chessnut-worker']:
                path = stage / 'runtime' / name
                path.write_bytes(b'foreign asset')
                (stage / 'SHA256SUMS').write_text(''.join(
                    f'{self.w.sha256(p)}  {p.relative_to(stage)}\n' for p in sorted(stage.rglob('*'))
                    if p.is_file() and p.name != 'SHA256SUMS'))
                with self.assertRaisesRegex(ValueError, 'Play.*assets'):
                    verifier().verify_artifacts('play', directory)
                path.unlink()

    def test_install_update_uninstall_preserve_config_and_other_app_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            root = base / 'kindle-chess-play'
            root.mkdir()
            config = root / 'config.json'
            config.write_bytes(b'{"future":99}')
            other_files = []
            for app in ['kindle-chess', 'kindle-game-review']:
                folder = base / app
                folder.mkdir()
                path = folder / 'user-data'
                path.write_bytes(app.encode())
                other_files.append(path)
            stage = self.base / 'stage'
            for _ in range(2):
                subprocess.run(['sh', str(stage / 'install.sh'), str(root)], check=True, capture_output=True)
            self.assertEqual((root / '.kindle-app-id').read_text().strip(), 'play')
            self.assertTrue((base / 'documents/kindle-chess-play.sh').exists())
            wrong = base / 'kindle-chess'
            (wrong / '.kindle-app-id').write_text('puzzles\n')
            result = subprocess.run(['sh', str(stage / 'install.sh'), str(wrong)], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            subprocess.run(['sh', str(stage / 'uninstall.sh'), str(root)], check=True, capture_output=True)
            self.assertEqual(config.read_bytes(), b'{"future":99}')
            self.assertFalse((root / 'runtime').exists())
            self.assertFalse((base / 'documents/kindle-chess-play.sh').exists())
            for path in other_files:
                self.assertEqual(path.read_bytes(), path.parent.name.encode())

if __name__ == '__main__':
    unittest.main()

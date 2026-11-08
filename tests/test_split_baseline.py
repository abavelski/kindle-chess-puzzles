"""Freeze installed Puzzles identity while product code is extracted."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SplitBaselineTests(unittest.TestCase):
    def test_legacy_package_identity(self):
        manifest = json.loads((ROOT / 'packaging/kindle/manifest.json').read_text())
        self.assertEqual(manifest['id'], 'kindle_chess')
        scriptlet = (ROOT / 'packaging/kindle/kindle-chess.sh').read_text()
        self.assertIn('/mnt/us/kindle-chess', scriptlet)
        self.assertTrue((ROOT / 'app/kindle-chess/Cargo.toml').is_file())


if __name__ == '__main__':
    unittest.main()

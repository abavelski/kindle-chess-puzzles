"""Enforce the domain-free device boundary during product extraction."""
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class PlatformBoundaryTests(unittest.TestCase):
    def test_platform_has_no_product_dependency_or_import(self):
        crate = ROOT / 'crates/kindle-platform'
        manifest = tomllib.loads((crate / 'Cargo.toml').read_text())
        self.assertFalse({'chess-core', 'chess-render', 'puzzle-app', 'review-app'} & manifest['dependencies'].keys())
        for source in (crate / 'src').glob('*.rs'):
            self.assertNotIn('chess_core', source.read_text(), source.name)
            self.assertNotIn('chess_render', source.read_text(), source.name)


if __name__ == '__main__':
    unittest.main()

"""CI must select and publish both independent products."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class MatrixContractTests(unittest.TestCase):
    def test_ci_builds_stages_verifies_and_publishes_each_product(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        for app in ['puzzles', 'review']:
            self.assertIn('app: ' + app, workflow)
        for fragment in ['scripts/app.sh build ${{ matrix.app }}',
                         'scripts/app.sh stage ${{ matrix.app }}',
                         'tools/verify_app_artifacts.py --app ${{ matrix.app }}',
                         'target/kindle/${{ matrix.app }}/',
                         'if-no-files-found: error']:
            self.assertIn(fragment, workflow)
        self.assertIn('name: kindle-${{ matrix.app }}-armv7-glibc235', workflow)
        self.assertLess(workflow.index('tools/verify_app_artifacts.py'), workflow.index('actions/upload-artifact'))

# The positive fixture is a complete Review stage/archive pair; mutations prove
# the same guard used before CI uploads fails on missing/substituted artifacts.
import importlib.util
import json
import shutil
import tarfile
import tempfile
from tests.test_packaging import workflow, elf


def verifier():
    spec = importlib.util.spec_from_file_location('verify_artifacts', ROOT / 'tools/verify_app_artifacts.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ArtifactGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.original = tempfile.TemporaryDirectory()
        cls.base = Path(cls.original.name)
        cls.w = workflow()
        spec = cls.w.product('review')
        binary = cls.base / spec['binary']
        binary.write_bytes(elf('review'))
        receipt = {'app': 'review', **{key: spec[key] for key in ['binary', 'package_id', 'runtime_root']},
                   'binary_sha256': cls.w.sha256(binary), 'source_sha256': cls.w.source_digest('review'),
                   'checks': 'passed', 'target': cls.w.TARGET, 'rust': '1.85.1',
                   'zig': cls.w.ZIG_VERSION, 'glibc': '2.35', 'fbink': cls.w.FBINK}
        (cls.base / 'build.json').write_text(json.dumps(receipt))
        cls.w.stage(binary, cls.base / 'build.json', cls.base / 'stage', app='review')
        cls.w.archive(cls.base / 'stage', cls.base / 'kindle-game-review.tar')
        cls.w.kpkg(cls.base / 'stage', cls.base / 'kindle_game_review_0.1.0_kindlehf.kpkg')

    @classmethod
    def tearDownClass(cls):
        cls.original.cleanup()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name) / 'review'
        shutil.copytree(self.base, self.directory)
        self.checker = verifier()

    def verify(self):
        return self.checker.verify_artifacts('review', self.directory)

    def test_complete_selected_artifacts_pass(self):
        self.assertEqual(self.verify()['app'], 'review')

    def test_each_required_artifact_is_mandatory(self):
        for name in ['kindle-game-review', 'build.json', 'stage/manifest.json',
                     'kindle-game-review.tar', 'kindle_game_review_0.1.0_kindlehf.kpkg']:
            path = self.directory / name
            saved = path.with_name(path.name + '.saved')
            path.rename(saved)
            with self.subTest(name=name), self.assertRaises((OSError, ValueError)):
                self.verify()
            saved.rename(path)
        self.verify()

    def test_wrong_app_selection_and_receipt_are_rejected(self):
        with self.assertRaises((OSError, ValueError)):
            self.checker.verify_artifacts('puzzles', self.directory)
        path = self.directory / 'build.json'
        receipt = json.loads(path.read_text())
        receipt['source_sha256'] = 'different source'
        path.write_text(json.dumps(receipt))
        with self.assertRaises(ValueError): self.verify()

    def test_review_rejects_runtime_assets_even_with_updated_stage_checksums(self):
        stage = self.directory / 'stage'
        for name in ['puzzles.json', 'kindlebt-worker']:
            path = stage / 'runtime' / name
            path.write_bytes(b'unexpected product asset')
            sums = ''.join(f'{self.w.sha256(p)}  {p.relative_to(stage)}\n' for p in sorted(stage.rglob('*'))
                           if p.is_file() and p.name != 'SHA256SUMS')
            (stage / 'SHA256SUMS').write_text(sums)
            with self.assertRaisesRegex(ValueError, 'Review.*assets'): self.verify()
            path.unlink()
            sums = ''.join(f'{self.w.sha256(p)}  {p.relative_to(stage)}\n' for p in sorted(stage.rglob('*'))
                           if p.is_file() and p.name != 'SHA256SUMS')
            (stage / 'SHA256SUMS').write_text(sums)
            self.verify()

    def test_corrupt_archive_cannot_be_published(self):
        (self.directory / 'kindle-game-review.tar').write_bytes(b'corrupt archive')
        with self.assertRaises(tarfile.TarError): self.verify()


if __name__ == '__main__':
    unittest.main()

"""Build/package contracts; install tests run only in temporary directories."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import struct
import subprocess
import tarfile
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]


def workflow():
    spec = importlib.util.spec_from_file_location('kindle_build', ROOT / 'tools/kindle_build.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def elf(app='puzzles'):
    data = bytearray(52)
    data[:7] = b'\x7fELF\x01\x01\x01'
    struct.pack_into('<H', data, 18, 40)
    struct.pack_into('<I', data, 36, 0x05000400)
    return bytes(data) + b'/lib/ld-linux-armhf.so.3\x00GLIBC_2.35\x00' + f'KINDLE_CHESS_PRODUCT:{app}:v1'.encode()


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.w = workflow()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.binary = self.base / 'kindle-chess'
        self.binary.write_bytes(elf())
        self.receipt = self.base / 'build.json'
        spec = importlib.util.spec_from_file_location('board_elf_fixture', ROOT / 'support/chessnut-move/tests/test_build_tools.py')
        fixture = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(fixture)
        bundle = self.base / 'chessnut'
        bundle.mkdir()
        (bundle / 'kindlebt-worker').write_bytes(fixture.elf_fixture())
        (bundle / 'libkindlebt.so').write_bytes(fixture.elf_fixture(False))
        (bundle / 'chessnut-required-libs.txt').write_text('libc.so.6\n')
        board = self.w.board_build()
        self.chessnut = {'artifacts': {p.name: self.w.sha256(p) for p in bundle.iterdir()},
                        'elf': {name: board.helpers.inspect_elf((bundle / name).read_bytes(), name == 'kindlebt-worker')
                                for name in ['kindlebt-worker', 'libkindlebt.so']}}
        self.receipt.write_text(json.dumps({'app': 'puzzles', 'binary': 'kindle-chess', 'package_id': 'kindle_chess', 'runtime_root': '/mnt/us/kindle-chess', 'chessnut': self.chessnut, 'binary_sha256': self.w.sha256(self.binary),
                                           'source_sha256': self.w.source_digest(),
                                           'target': self.w.TARGET, 'checks': 'passed'}))
        self.stage = self.base / 'stage'

    def test_review_stage_is_independent_and_rejects_cross_app_substitution(self):
        review_binary = self.base / 'review-binary'
        review_binary.write_bytes(elf('review'))
        receipt = {'app': 'review', 'binary': 'kindle-game-review',
                   'package_id': 'kindle_game_review', 'runtime_root': '/mnt/us/kindle-game-review',
                   'binary_sha256': self.w.sha256(review_binary),
                   'source_sha256': self.w.source_digest('review'), 'target': self.w.TARGET, 'checks': 'passed'}
        review_receipt = self.base / 'review.json'
        review_receipt.write_text(json.dumps(receipt))
        review_stage = self.base / 'review-stage'
        self.w.stage(review_binary, review_receipt, review_stage, app='review')
        self.w.verify_stage(review_stage, app='review')
        self.assertFalse((review_stage / 'runtime/kindlebt-worker').exists())
        self.assertFalse((review_stage / 'runtime/lib/libkindlebt.so').exists())
        self.assertFalse((review_stage / 'runtime/kindle-chess').exists())
        self.assertEqual(json.loads((review_stage / 'manifest.json').read_text())['id'], 'kindle_game_review')
        with self.assertRaises(ValueError):
            self.w.stage(review_binary, review_receipt, self.base / 'wrong-stage', app='puzzles')
        with self.assertRaises(ValueError):
            self.w.verify_stage(review_stage, app='puzzles')

    def test_receipt_cannot_relabel_another_products_binary(self):
        self.binary.write_bytes(elf('review'))
        receipt = json.loads(self.receipt.read_text())
        receipt['binary_sha256'] = self.w.sha256(self.binary)
        self.receipt.write_text(json.dumps(receipt))
        with self.assertRaises(ValueError): self.stage_build()
        self.assertNotEqual(self.w.output_dir('puzzles'), self.w.output_dir('review'))
        self.assertNotEqual(self.w.source_digest('puzzles'), self.w.source_digest('review'))

    def test_coinstall_update_uninstall_preserve_the_other_product_and_its_data(self):
        self.stage_build()
        binary = self.base / 'review-binary'
        binary.write_bytes(elf('review'))
        receipt_path = self.base / 'review.json'
        receipt = {'app': 'review', 'binary': 'kindle-game-review',
                   'package_id': 'kindle_game_review', 'runtime_root': '/mnt/us/kindle-game-review',
                   'binary_sha256': self.w.sha256(binary), 'source_sha256': self.w.source_digest('review'),
                   'target': self.w.TARGET, 'checks': 'passed'}
        receipt_path.write_text(json.dumps(receipt))
        review_stage = self.base / 'review-stage'
        self.w.stage(binary, receipt_path, review_stage, app='review')
        roots = [self.base / 'puzzles-root', self.base / 'review-root']
        stages = [self.stage, review_stage]
        scriptlets = ['kindle-chess.sh', 'kindle-game-review.sh']
        def install(stage, root):
            return subprocess.run(['sh', str(stage / 'install.sh'), str(root)], capture_output=True, text=True)
        def snapshot(root):
            return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in root.rglob('*') if p.is_file()}
        for stage, root in zip(stages, roots):
            for directory in ['puzzles', 'games', 'state', 'logs']:
                (root / directory).mkdir(parents=True)
                (root / directory / 'user-file').write_bytes(b'opaque user data')
            result = install(stage, root)
            self.assertEqual(result.returncode, 0, result.stderr)
        for selected in [0, 1]:
            other = 1 - selected
            before = snapshot(roots[other])
            document = self.base / 'documents' / scriptlets[other]
            document_bytes = document.read_bytes()
            artifact = self.binary if selected == 0 else binary
            attestation = self.receipt if selected == 0 else receipt_path
            artifact.write_bytes(artifact.read_bytes() + b' updated product')
            updated = json.loads(attestation.read_text())
            updated['binary_sha256'] = self.w.sha256(artifact)
            attestation.write_text(json.dumps(updated))
            self.w.stage(artifact, attestation, stages[selected], app='puzzles' if selected == 0 else 'review')
            self.assertEqual(install(stages[selected], roots[selected]).returncode, 0)
            # Even an explicitly misrouted root must not overwrite the other app.
            self.assertNotEqual(install(stages[selected], roots[other]).returncode, 0)
            wrong_uninstall = subprocess.run(['sh', str(stages[selected] / 'uninstall.sh'), str(roots[other])], capture_output=True)
            self.assertNotEqual(wrong_uninstall.returncode, 0)
            subprocess.run(['sh', str(stages[selected] / 'uninstall.sh'), str(roots[selected])], check=True, capture_output=True)
            self.assertEqual(snapshot(roots[other]), before)
            self.assertEqual(document.read_bytes(), document_bytes)
            for directory in ['puzzles', 'games', 'state', 'logs']:
                self.assertEqual((roots[selected] / directory / 'user-file').read_bytes(), b'opaque user data')
            self.assertEqual(install(stages[selected], roots[selected]).returncode, 0)
        with mock.patch.object(self.w, 'run') as remote:
            self.w.deploy(review_stage, 'root@device', 2222, app='review')
            command = remote.call_args_list[-1].args[0][-1]
            self.assertIn('/mnt/us/kindle-game-review/runtime/source.tar', command)
            self.assertIn('sha256sum -c -', command)
            self.assertNotIn('/mnt/us/kindle-chess/', command)
        before = snapshot(roots[1])
        (review_stage / 'runtime/kindle-game-review').write_bytes(b'corrupt payload')
        self.assertNotEqual(install(review_stage, roots[1]).returncode, 0)
        self.assertEqual(snapshot(roots[1]), before)
        bad_archive = self.base / 'corrupt.tar'
        bad_archive.write_bytes(b'corrupt archive bytes')
        incoming = self.base / 'incoming'
        incoming.mkdir()
        failed = subprocess.run(['sh', '-c', 'tar xf "$1" && sh ./install.sh "$2"',
                                 'sh', str(bad_archive), str(roots[1])], cwd=incoming, capture_output=True)
        self.assertNotEqual(failed.returncode, 0)
        self.assertEqual(snapshot(roots[1]), before)


    def test_board_bundle_checksums_and_private_sources(self):
        self.stage_build()
        self.assertTrue((self.stage / 'runtime/kindlebt-worker').stat().st_mode & 0o111)
        self.assertTrue((self.stage / 'runtime/lib/libkindlebt.so').is_file())
        self.assertFalse((self.stage / 'runtime/lib/libace_bt.so').exists())
        with tarfile.open(self.stage / 'runtime/source.tar') as source:
            self.assertIn('support/chessnut-move/native/kindlebt_worker.c', source.getnames())
            self.assertFalse(any(name.endswith('.private.json') or name.startswith('local-kindle/') for name in source.getnames()))
        (self.stage / 'runtime/lib/libkindlebt.so').write_bytes(b'corrupt')
        with self.assertRaises(ValueError): self.w.verify_stage(self.stage)

    def stage_build(self):
        self.w.stage(self.binary, self.receipt, self.stage)

    def test_artifact_rejects_wrong_arch_soft_float_and_new_libc(self):
        self.w.validate_elf(self.binary)
        for data in [b'not an executable', elf().replace(b'GLIBC_2.35', b'GLIBC_2.38'),
                     elf()[:36] + struct.pack('<I', 0x05000200) + elf()[40:]]:
            self.binary.write_bytes(data)
            with self.assertRaises(ValueError):
                self.w.validate_elf(self.binary)

    def test_stage_refuses_to_replace_an_unrelated_directory(self):
        self.stage.mkdir()
        (self.stage / 'user-file').write_text('preserve me')
        with self.assertRaises(ValueError):
            self.stage_build()
        self.assertEqual((self.stage / 'user-file').read_text(), 'preserve me')

    def test_corresponding_source_verifies_without_git_and_excludes_build_outputs(self):
        self.stage_build()
        extracted = self.base / 'source'
        extracted.mkdir()
        with tarfile.open(self.stage / 'runtime/source.tar') as source:
            self.assertNotIn('vendor/FBInk/libi2c.built', source.getnames())
            self.assertIn('.cargo/config.toml', source.getnames())
            source.extractall(extracted, filter='data')
        subprocess.run(['cargo', 'metadata', '--offline', '--locked', '--format-version=1'],
                       cwd=extracted, check=True, capture_output=True)
        with mock.patch.object(self.w, 'ROOT', extracted):
            self.w.verify_vendor()
            # FBInk make creates ignored libi2c products even for Kindle.
            (extracted / 'vendor/FBInk/libi2c.built').write_text('generated')
            self.w.verify_vendor()
            (extracted / 'vendor/FBInk/fbink.h').write_text('changed')
            with self.assertRaises(ValueError):
                self.w.verify_vendor()

    def test_stage_requires_successful_checks_for_exact_source_and_binary(self):
        for field, value in [('checks', 'failed'), ('source_sha256', 'stale'),
                             ('binary_sha256', 'wrong')]:
            original = self.receipt.read_text()
            receipt = json.loads(original)
            receipt[field] = value
            self.receipt.write_text(json.dumps(receipt))
            with self.assertRaises(ValueError):
                self.stage_build()
            self.assertFalse(self.stage.exists())
            self.receipt.write_text(original)

    def test_stage_and_archive_are_deterministic_and_detect_tampering(self):
        self.stage_build()
        self.w.verify_stage(self.stage)
        first = self.base / 'first.tar'
        second = self.base / 'second.tar'
        self.w.archive(self.stage, first)
        self.w.archive(self.stage, second)
        self.assertEqual(first.read_bytes(), second.read_bytes())
        self.assertTrue((self.stage / 'runtime/kindle_launch.sh').exists())
        package = self.base / 'kindle-chess.kpkg'
        self.w.kpkg(self.stage, package)
        with tarfile.open(package, 'r:gz') as packed:
            manifest = json.load(packed.extractfile('manifest.json'))
            self.assertEqual(manifest['supported_platforms'], ['kindlehf'])
            self.assertIsNotNone(packed.extractfile('launch.sh'))
        self.assertFalse((self.stage / 'state').exists())
        self.assertFalse((self.stage / 'puzzles').exists())
        (self.stage / 'runtime/kindle-chess').write_bytes(b'tampered')
        with self.assertRaises(ValueError):
            self.w.verify_stage(self.stage)

    def install(self, root):
        return subprocess.run(['sh', str(self.stage / 'install.sh'), str(root)],
                              cwd=self.stage, capture_output=True, text=True)

    def test_reinstall_update_uninstall_preserve_all_user_data(self):
        self.stage_build()
        root = self.base / 'installed'
        for directory in ['puzzles', 'state', 'logs']:
            (root / directory).mkdir(parents=True)
            (root / directory / 'user-file').write_bytes(b'user data')
        for _ in range(2):
            result = self.install(root)
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((root / 'runtime/kindle-chess').read_bytes(), elf())
        self.binary.write_bytes(elf() + b'second build')
        receipt = json.loads(self.receipt.read_text())
        receipt['binary_sha256'] = self.w.sha256(self.binary)
        self.receipt.write_text(json.dumps(receipt))
        self.stage_build()
        result = self.install(root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((root / 'runtime/kindle-chess').read_bytes().endswith(b'second build'))
        subprocess.run(['sh', str(self.stage / 'uninstall.sh'), str(root)], check=True)
        self.assertFalse((root / 'runtime').exists())
        self.assertFalse((root.parent / 'documents/kindle-chess.sh').exists())
        for directory in ['puzzles', 'state', 'logs']:
            self.assertEqual((root / directory / 'user-file').read_bytes(), b'user data')
        self.assertEqual(self.install(root).returncode, 0)

    def test_install_refuses_busy_or_corrupt_runtime_without_changing_old_install(self):
        self.stage_build()
        root = self.base / 'installed'
        self.assertEqual(self.install(root).returncode, 0)
        (root / '.install-lock').mkdir()
        self.assertNotEqual(self.install(root).returncode, 0)
        (root / '.install-lock').rmdir()
        (self.stage / 'runtime/kindle-chess').write_bytes(b'corrupt')
        self.assertNotEqual(self.install(root).returncode, 0)
        self.assertEqual((root / 'runtime/kindle-chess').read_bytes(), elf())

    def test_scriptlet_delegates_to_runtime_and_owns_framebuffer_output(self):
        self.stage_build()
        scriptlet = self.stage / 'kindle-chess.sh'
        # SH_Integration consumes this directive, preventing a second FBInk writer.
        self.assertIn('# DontUseFBInk', scriptlet.read_text().splitlines())
        commands = self.base / 'commands'
        commands.mkdir()
        stub = commands / 'sh'
        stub.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$CAPTURE"\n')
        stub.chmod(0o755)
        capture = self.base / 'invoked'
        subprocess.run(['/bin/sh', str(scriptlet)], check=True,
                       env=dict(os.environ, PATH=str(commands), CAPTURE=str(capture)))
        self.assertEqual(capture.read_text().splitlines(), ['/mnt/us/kindle-chess/runtime/launch.sh'])

    def test_install_creates_scriptlet_but_preserves_foreign_document(self):
        self.stage_build()
        root = self.base / 'installed'
        document = root.parent / 'documents/kindle-chess.sh'
        self.assertEqual(self.install(root).returncode, 0)
        self.assertEqual(document.read_bytes(), (self.stage / 'kindle-chess.sh').read_bytes())
        document.write_text('someone else owns this document')
        self.assertNotEqual(self.install(root).returncode, 0)
        self.assertEqual(document.read_text(), 'someone else owns this document')
        subprocess.run(['sh', str(self.stage / 'uninstall.sh'), str(root)], check=True)
        self.assertEqual(document.read_text(), 'someone else owns this document')

    def test_verified_pre_marker_stage_can_upgrade_but_cannot_publish_as_new(self):
        self.stage_build()
        (self.stage / 'runtime/app-id').unlink()
        sums = self.stage / 'SHA256SUMS'
        sums.write_text(''.join(line + '\n' for line in sums.read_text().splitlines() if not line.endswith('  runtime/app-id')))
        with self.assertRaises((ValueError, OSError)): self.w.verify_stage(self.stage)
        self.stage_build()
        self.w.verify_stage(self.stage)
        self.assertEqual((self.stage / 'runtime/app-id').read_text().strip(), 'puzzles')

    def test_legacy_upgrade_failed_scriptlet_swap_and_unknown_runtime_recovery(self):
        self.stage_build()
        root = self.base / 'legacy-root'
        (root / 'runtime').mkdir(parents=True)
        (root / 'runtime/kindle-chess').write_bytes(b'pre-split combined runtime')
        (root / 'state').mkdir()
        (root / 'state/progress.json').write_bytes(b'future progress bytes')
        self.assertEqual(self.install(root).returncode, 0)
        self.assertEqual((root / '.kindle-app-id').read_text().strip(), 'puzzles')
        before = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}
        binary = self.binary.read_bytes() + b' updated'
        self.binary.write_bytes(binary)
        receipt = json.loads(self.receipt.read_text())
        receipt['binary_sha256'] = self.w.sha256(self.binary)
        self.receipt.write_text(json.dumps(receipt))
        self.stage_build()
        commands = self.base / 'commands'
        commands.mkdir()
        (commands / 'mv').write_text('#!/bin/sh\n[ "$2" != "$FAIL_DOCUMENT" ] || exit 9\nexec /bin/mv "$@"\n')
        (commands / 'mv').chmod(0o755)
        result = subprocess.run(['sh', str(self.stage / 'install.sh'), str(root)], capture_output=True,
                                env=dict(os.environ, PATH=str(commands) + os.pathsep + os.environ['PATH'],
                                         FAIL_DOCUMENT=str(root.parent / 'documents/kindle-chess.sh')))
        self.assertNotEqual(result.returncode, 0)
        after = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}
        self.assertEqual(after, before, 'failed final swap restores the prior complete runtime')
        unknown = root / 'runtime.new'
        unknown.mkdir()
        (unknown / 'user-file').write_bytes(b'unknown bytes')
        self.assertNotEqual(self.install(root).returncode, 0)
        self.assertEqual((unknown / 'user-file').read_bytes(), b'unknown bytes')
        (unknown / 'user-file').unlink()
        unknown.rmdir()
        self.assertEqual(self.install(root).returncode, 0)
        self.assertEqual((root / 'state/progress.json').read_bytes(), b'future progress bytes')

    def test_launcher_logs_architecture_failure_before_running_binary(self):
        self.stage_build()
        root = self.base / 'installed'
        self.assertEqual(self.install(root).returncode, 0)
        result = subprocess.run(['sh', str(root / 'runtime/launch.sh')], capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('armv7l', (root / 'logs/launch.log').read_text())

    def test_deploy_rejects_tampering_before_network_and_unsafe_destinations(self):
        self.stage_build()
        for host in ['-oProxyCommand=evil', 'root@host;echo bad']:
            with self.assertRaises(ValueError):
                self.w.deploy(self.stage, host, 2222)
        (self.stage / 'runtime/kindle-chess').write_bytes(b'tampered')
        with self.assertRaises(ValueError):
            self.w.deploy(self.stage, 'root@192.168.1.20', 2222)


class BuildGateTests(unittest.TestCase):
    def test_failed_checks_never_cross_build_or_leave_receipt(self):
        w = workflow()
        with tempfile.TemporaryDirectory() as temporary:
            build_dir = Path(temporary)
            (build_dir / 'puzzles').mkdir()
            (build_dir / 'puzzles/build.json').write_text('old receipt')
            with mock.patch.object(w, 'BUILD', build_dir), mock.patch.object(w, 'verify_vendor'), \
                    mock.patch.object(w, 'run', side_effect=subprocess.CalledProcessError(1, 'checks')) as run, \
                    mock.patch.object(w, 'zig_tools') as zig:
                with self.assertRaises(subprocess.CalledProcessError):
                    w.build()
                self.assertEqual(run.call_args.args[0], ['sh', 'scripts/check.sh'])
                zig.assert_not_called()
                self.assertFalse((build_dir / 'puzzles/build.json').exists())

    def test_cross_bridge_rebuilds_when_toolchain_changes(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / 'build-script'
            subprocess.run(['rustc', '--edition=2021', str(ROOT / 'crates/fbink-sys/build.rs'), '-o', str(binary)], check=True)
            output = subprocess.run([str(binary)], env=dict(os.environ, TARGET='host-contract'),
                                    check=True, capture_output=True, text=True).stdout
        for variable in ['KINDLE_CC', 'KINDLE_AR', 'KINDLE_RANLIB']:
            self.assertIn(f'cargo:rerun-if-env-changed={variable}', output)

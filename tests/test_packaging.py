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


def elf():
    data = bytearray(52)
    data[:7] = b'\x7fELF\x01\x01\x01'
    struct.pack_into('<H', data, 18, 40)
    struct.pack_into('<I', data, 36, 0x05000400)
    return bytes(data) + b'/lib/ld-linux-armhf.so.3\x00GLIBC_2.35\x00'


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.w = workflow()
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.binary = self.base / 'kindle-chess'
        self.binary.write_bytes(elf())
        self.receipt = self.base / 'build.json'
        self.receipt.write_text(json.dumps({'binary_sha256': self.w.sha256(self.binary),
                                           'source_sha256': self.w.source_digest(),
                                           'target': self.w.TARGET, 'checks': 'passed'}))
        self.stage = self.base / 'stage'

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
            (build_dir / 'build.json').write_text('old receipt')
            with mock.patch.object(w, 'BUILD', build_dir), mock.patch.object(w, 'verify_vendor'), \
                    mock.patch.object(w, 'run', side_effect=subprocess.CalledProcessError(1, 'checks')) as run, \
                    mock.patch.object(w, 'zig_tools') as zig:
                with self.assertRaises(subprocess.CalledProcessError):
                    w.build()
                self.assertEqual(run.call_args.args[0], ['sh', 'scripts/check.sh'])
                zig.assert_not_called()
                self.assertFalse((build_dir / 'build.json').exists())

    def test_cross_bridge_rebuilds_when_toolchain_changes(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / 'build-script'
            subprocess.run(['rustc', '--edition=2021', str(ROOT / 'crates/fbink-sys/build.rs'), '-o', str(binary)], check=True)
            output = subprocess.run([str(binary)], env=dict(os.environ, TARGET='host-contract'),
                                    check=True, capture_output=True, text=True).stdout
        for variable in ['KINDLE_CC', 'KINDLE_AR', 'KINDLE_RANLIB']:
            self.assertIn(f'cargo:rerun-if-env-changed={variable}', output)

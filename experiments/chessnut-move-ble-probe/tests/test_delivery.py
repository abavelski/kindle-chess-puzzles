"""Host delivery contracts; never contacts SSH or a board."""
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('delivery', ROOT / 'scripts/delivery.py')
delivery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(delivery)

class ElfTests(unittest.TestCase):
    def test_rejects_host_or_truncated_binary(self):
        for data in (b'', b'not ELF', b'\x7fELF\x02\x01\x01' + bytes(200)):
            with self.assertRaises(ValueError):
                delivery.inspect_elf(data, executable=True)

    def test_rejects_wrong_architecture_float_loader_and_new_glibc(self):
        good = elf_fixture()
        self.assertEqual(delivery.inspect_elf(good, executable=True)['needed'], ['libc.so.6'])
        for offset, value in [(18, struct.pack('<H', 62)), (36, struct.pack('<I', 0x05000200))]:
            data = bytearray(good)
            data[offset:offset+len(value)] = value
            with self.assertRaises(ValueError): delivery.inspect_elf(data, executable=True)
        for old, new in [(b'/lib/ld-linux-armhf.so.3', b'/lib/ld-linux-wrong.so.3'), (b'GLIBC_2.35', b'GLIBC_2.38')]:
            with self.assertRaises(ValueError): delivery.inspect_elf(good.replace(old, new), executable=True)

    def test_shared_library_needs_no_interpreter(self):
        self.assertEqual(delivery.inspect_elf(elf_fixture(False), executable=False)['interpreter'], None)

class ManifestTests(unittest.TestCase):
    def test_manifest_rejects_tampering_extra_files_and_symlinks(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'payload').write_text('good')
            delivery.write_manifest(root)
            delivery.verify_manifest(root)
            (root / 'payload').write_text('bad')
            with self.assertRaises(ValueError): delivery.verify_manifest(root)
            (root / 'payload').write_text('good')
            (root / 'extra').write_text('bad')
            with self.assertRaises(ValueError): delivery.verify_manifest(root)
            (root / 'extra').unlink()
            (root / 'payload').unlink()
            (root / 'payload').symlink_to('/etc/passwd')
            with self.assertRaises(ValueError): delivery.verify_manifest(root)

    def test_manifest_rejects_traversal(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'SHA256SUMS').write_text('0'*64 + '  ../outside\n')
            with self.assertRaises(ValueError): delivery.verify_manifest(root)

def elf_fixture(executable=True):
    data = bytearray(1024)
    data[:16] = b'\x7fELF\x01\x01\x01' + bytes(9)
    struct.pack_into('<HHIIIIIHHHHHH', data, 16, 3, 40, 1, 0, 52, 0, 0x05000400, 52, 32, 3 if executable else 2, 0, 0, 0)
    struct.pack_into('<IIIIIIII', data, 52, 1, 0, 0x1000, 0, len(data), len(data), 5, 4096)
    struct.pack_into('<IIIIIIII', data, 84, 2, 256, 0x1100, 0, 64, 64, 4, 4)
    if executable:
        struct.pack_into('<IIIIIIII', data, 116, 3, 400, 0x1190, 0, 24, 24, 4, 1)
        data[400:424] = b'/lib/ld-linux-armhf.so.3\0'
    strings = b'\0libc.so.6\0GLIBC_2.35\0'
    data[512:512+len(strings)] = strings
    for index, (tag, value) in enumerate([(5, 0x1200), (10, len(strings)), (1, 1), (0x6ffffffe, 0x1300), (0x6fffffff, 1), (0, 0)]):
        struct.pack_into('<II', data, 256+index*8, tag, value)
    struct.pack_into('<HHIII', data, 768, 1, 1, 1, 16, 0)
    struct.pack_into('<IHHII', data, 784, 0, 0, 1, 11, 0)
    return bytes(data)


class InstallerTests(unittest.TestCase):
    def setUp(self):
        import shutil
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name).resolve()
        self.root = self.base / 'chessnut-ble-probe'
        self.root.mkdir()
        (self.root / '.probe-owner').write_text('chessnut-ble-probe-v1\n')
        self.main = self.base / 'kindle-chess'
        self.main.mkdir()
        (self.main / 'progress.json').write_text('preserve me')
        self.incoming = self.root / 'incoming-test'
        self.incoming.mkdir()
        runtime = self.incoming / 'runtime'
        runtime.mkdir()
        (runtime / 'payload').write_text('new')
        delivery.write_manifest(runtime)
        self.script = self.incoming / 'install.sh'
        self.script.write_text((ROOT / 'scripts/install-device.sh').read_text().replace('/mnt/us/chessnut-ble-probe', str(self.root)))
        delivery.write_manifest(self.incoming)

    def tearDown(self): self.tmp.cleanup()

    def install(self, **kwargs):
        import subprocess
        return subprocess.run(['sh', str(self.script)], cwd=self.incoming, capture_output=True, **kwargs)

    def test_install_update_and_scoped_uninstall(self):
        import subprocess
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual((self.root / 'runtime/payload').read_text(), 'new')
        (self.root / 'runtime/payload').write_text('old')
        self.assertEqual(self.install().returncode, 0)
        self.assertFalse((self.root / 'runtime.old').exists())
        script = self.base / 'uninstall.sh'
        script.write_text((ROOT / 'scripts/uninstall-device.sh').read_text().replace('/mnt/us/chessnut-ble-probe', str(self.root)))
        self.assertEqual(subprocess.run(['sh', script], capture_output=True).returncode, 0)
        self.assertFalse(self.root.exists())
        self.assertEqual((self.main / 'progress.json').read_text(), 'preserve me')

    def test_rejects_bad_checksum_symlink_and_active_lock(self):
        (self.incoming / 'runtime/payload').write_text('tampered')
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse((self.root / 'runtime').exists())
        (self.incoming / 'runtime/payload').write_text('new')
        (self.root / '.operation-lock').mkdir()
        self.assertNotEqual(self.install().returncode, 0)
        (self.root / '.operation-lock').rmdir()
        (self.root / 'runtime').symlink_to(self.main, target_is_directory=True)
        self.assertNotEqual(self.install().returncode, 0)
        self.assertTrue((self.main / 'progress.json').exists())

    def test_failed_swap_restores_previous_runtime(self):
        import os
        self.assertEqual(self.install().returncode, 0)
        (self.root / 'runtime/payload').write_text('old')
        wrappers = self.base / 'bin'
        wrappers.mkdir()
        import shutil
        real_mv = shutil.which('mv')
        wrapper = wrappers / 'mv'
        wrapper.write_text(f'#!/bin/sh\ncase "$1" in */runtime.new) exit 9;; esac\nexec {real_mv} "$@"\n')
        wrapper.chmod(0o755)
        result = self.install(env=dict(os.environ, PATH=str(wrappers)+':'+os.environ['PATH']))
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.root / 'runtime/payload').read_text(), 'old')
        self.assertFalse((self.root / '.operation-lock').exists())

class NeededTests(unittest.TestCase):
    def test_only_the_pinned_sdk_reference_can_be_localized(self):
        original = bytearray(elf_fixture())
        absolute = b'/build/sdk/libace_bt.so'
        strings = b'\0' + absolute + b'\0GLIBC_2.35\0'
        original[512:512+len(strings)] = strings
        struct.pack_into('<I', original, 268, len(strings))
        struct.pack_into('<I', original, 792, len(absolute)+2)
        with self.assertRaises(ValueError): delivery.inspect_elf(original, executable=True)
        fixed = delivery.localize_needed(bytes(original), '/build/sdk/libace_bt.so')
        self.assertEqual(delivery.inspect_elf(fixed, executable=True)['needed'], ['libace_bt.so'])
        with self.assertRaises(ValueError): delivery.localize_needed(bytes(original), '/different/libace_bt.so')


class RemoteTests(unittest.TestCase):
    def test_host_port_validation_happens_before_remote_execution(self):
        from types import SimpleNamespace
        self.assertIn('ConnectTimeout=15', delivery.remote_options(SimpleNamespace(host='root@device.example', port=2222)))
        for host, port in [('root@host;bad',2222), ('-F@host',2222), ('host',0), ('host',65536), ('',2222)]:
            with self.assertRaises(ValueError): delivery.remote_options(SimpleNamespace(host=host, port=port))


class DeployTests(unittest.TestCase):
    def test_legacy_scp_uploads_named_entries_not_dot_directory(self):
        import json
        from types import SimpleNamespace
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp)
            runtime = output / 'stage/runtime'
            runtime.mkdir(parents=True)
            (runtime / 'build.json').write_text(json.dumps({'source_sha256':'current'}))
            calls = []
            def fake_run(args, **kwargs):
                calls.append(args)
                if args[0] == 'scp':
                    self.assertFalse(any(str(arg).endswith('/.') for arg in args), 'Kindle legacy SCP rejects filename dot')
            with patch.object(delivery, 'OUTPUT', output), patch.object(delivery, 'verify_stage'), patch.object(delivery, 'source_digest', return_value='current'), patch.object(delivery, 'run', side_effect=fake_run):
                delivery.deploy(SimpleNamespace(host='root@device', port=2222))
            upload = next(args for args in calls if args[0] == 'scp')
            self.assertEqual(upload[-4:-1], [str(output/'stage/install.sh'), str(output/'stage/SHA256SUMS'), str(runtime)])


class RelocationTests(unittest.TestCase):
    def test_rejects_arm_copy_relocations_even_for_a_pie(self):
        data = bytearray(elf_fixture())
        # Extend PT_DYNAMIC with DT_REL/DT_RELSZ/DT_RELENT and one R_ARM_COPY.
        struct.pack_into('<II', data, 100, 80, 80)
        tags = [(5, 0x1200), (10, 22), (1, 1), (0x6ffffffe, 0x1300),
                (0x6fffffff, 1), (17, 0x1340), (18, 8), (19, 8), (0, 0)]
        for i, (tag, value) in enumerate(tags):
            struct.pack_into('<II', data, 256+i*8, tag, value)
        struct.pack_into('<II', data, 832, 0x1380, (1 << 8) | 20)
        with self.assertRaisesRegex(ValueError, 'COPY'):
            delivery.inspect_elf(bytes(data), executable=True)

    def test_rejects_fixed_address_native_executables(self):
        data = bytearray(elf_fixture())
        struct.pack_into('<H', data, 16, 2)  # ET_EXEC rather than ET_DYN (PIE)
        with self.assertRaisesRegex(ValueError, 'PIE'):
            delivery.inspect_elf(bytes(data), executable=True)


class NativeIdentityTests(unittest.TestCase):
    def test_daemon_identity_matches_observed_ids_not_passwd_primary_group(self):
        import subprocess
        with tempfile.TemporaryDirectory() as tmp:
            binary = Path(tmp) / 'identity-test'
            compiled = subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-I', str(ROOT / 'native'), str(ROOT / 'tests/native_identity_test.c'), '-o', str(binary)], capture_output=True)
            self.assertEqual(compiled.returncode, 0, compiled.stderr.decode())
            subprocess.run([str(binary)], check=True, timeout=5)

if __name__ == '__main__': unittest.main()

import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('build_tools',ROOT/'scripts/build_tools.py')
delivery=importlib.util.module_from_spec(spec)
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

class RuntimeDependencyTests(unittest.TestCase):
    def test_runtime_needs_come_from_shipped_artifacts_not_old_sdk_transitives(self):
        artifacts = {
            'controller': {'needed': ['ld-linux-armhf.so.3', 'libc.so.6']},
            'worker': {'needed': ['libc.so.6']},
            'libkindlebt.so': {'needed': ['libace_bt.so', 'libpthread.so.0', 'libc.so.6']},
        }
        self.assertEqual(delivery.runtime_dependencies(artifacts), ['ld-linux-armhf.so.3', 'libace_bt.so', 'libc.so.6', 'libpthread.so.0'])
        # A future genuinely linked dependency must still be checked.
        artifacts['worker']['needed'].append('libcrypto.so.3')
        self.assertIn('libcrypto.so.3', delivery.runtime_dependencies(artifacts))

class DescriptorBridgeTests(unittest.TestCase):
    def test_pinned_source_patch_only_adds_descriptor_forwarding(self):
        original = '#include "kindlebt_callbacks.c"\n' + (
            '    .on_ble_gattc_write_characteristics_cb = bleGattcWriteCharsCallbackWrapper,\n')
        patched = delivery.descriptor_bridge_source(original)
        self.assertIn('#include "kindlebt_descriptor_bridge.h"', patched)
        self.assertIn('.on_ble_gattc_write_descriptor_cb = probe_descriptor_callback_bridge,', patched)
        self.assertEqual(patched.replace('\n#include "kindlebt_descriptor_bridge.h"', '').replace(
            '    .on_ble_gattc_write_descriptor_cb = probe_descriptor_callback_bridge,\n', ''), original)
        for bad in ('', original + original, original.replace('bleGattcWriteCharsCallbackWrapper', 'changed')):
            with self.assertRaises(ValueError):
                delivery.descriptor_bridge_source(bad)

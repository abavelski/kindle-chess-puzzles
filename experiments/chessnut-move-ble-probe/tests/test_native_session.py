"""Portable production lifecycle/parser tests; no downloaded SDK or BLE required."""
from pathlib import Path
import subprocess
import tempfile
import unittest
ROOT = Path(__file__).resolve().parents[1]
class NativeSessionTests(unittest.TestCase):
    def test_native_lifecycle_and_strict_discovery_only_parser(self):
        with tempfile.TemporaryDirectory() as tmp:
            binary = Path(tmp) / 'session-test'
            result = subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-I', str(ROOT / 'native'), str(ROOT / 'tests/native_session_test.c'), '-o', str(binary)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(binary)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

class LauncherTests(unittest.TestCase):
    def test_busybox_background_input_uses_a_saved_descriptor(self):
        # On the Scribe's ash, a noninteractive background job receives null
        # stdin before <&0 is applied. A separate saved descriptor survives.
        script = (ROOT / 'scripts/launch.sh').read_text()
        saved = script.index('exec 3<&0') if 'exec 3<&0' in script else -1
        invocation = next(line for line in script.splitlines() if line.startswith('LD_LIBRARY_PATH='))
        self.assertGreaterEqual(saved, 0, 'preserve SSH input before ash starts a background process')
        self.assertLess(saved, script.index(invocation))
        self.assertIn('<&3 3<&- &', invocation, 'restore stdin and close the extra descriptor in the child')

    def test_session_forwards_arguments_and_stdin_and_releases_operation_lock(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'chessnut-ble-probe'
            runtime = root / 'runtime'
            runtime.mkdir(parents=True)
            (root / '.probe-owner').write_text('chessnut-ble-probe-v1\n')
            (runtime / 'SHA256SUMS').write_text('')
            binary = runtime / 'chessnut-ble-probe'
            binary.write_text('#!/bin/sh\nprintf "args=%s,%s,%s\\n" "$1" "$2" "$3"\nIFS= read -r line\nprintf "stdin=%s\\n" "$line"\n')
            binary.chmod(0o755)
            script = (ROOT / 'scripts/launch.sh').read_text().replace('/mnt/us/chessnut-ble-probe', str(root))
            script = script.replace('sha256sum -c SHA256SUMS', ': # host test only: no native payload')
            architecture = next(line for line in script.splitlines() if line.startswith('[ "$(uname'))
            script = script.replace(architecture, ': # host test only: platform boundary')
            launcher = root / 'launch-test.sh'
            launcher.write_text(script)
            result = subprocess.run(['sh', launcher, 'session', '--address', '12:34:56:78:90:ab'], input='quit\n', capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('args=session,--address,12:34:56:78:90:ab', result.stdout)
            self.assertIn('stdin=quit', result.stdout)
            self.assertFalse((root / '.operation-lock').exists())
            for args in [('set-fen',), ('session', '--wrong', 'private')]:
                result = subprocess.run(['sh', launcher, *args], capture_output=True, timeout=5)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((root / '.operation-lock').exists())

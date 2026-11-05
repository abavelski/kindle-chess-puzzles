"""Portable native contracts; never opens a Bluetooth session."""
from pathlib import Path
import subprocess,tempfile,unittest
ROOT=Path(__file__).resolve().parents[1]
class NativeContracts(unittest.TestCase):
    def test_identity_lifecycle_and_packet_bounds(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name in ['native_identity_test','native_session_test','native_m3_test']:
                with self.subTest(name=name):
                    binary=Path(tmp)/name
                    subprocess.run(['cc','-std=c11','-Wall','-Wextra','-Werror','-I',str(ROOT/'native'),str(ROOT/'tests'/f'{name}.c'),'-o',str(binary)],check=True,capture_output=True)
                    subprocess.run([str(binary)],check=True,capture_output=True)

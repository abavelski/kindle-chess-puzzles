from pathlib import Path
import subprocess,tempfile,unittest
ROOT=Path(__file__).resolve().parents[1]
class NativeM3Tests(unittest.TestCase):
    def test_native_packet_policy_parser_and_notification_bounds(self):
        with tempfile.TemporaryDirectory() as tmp:
            output=Path(tmp)/'m3-test'
            compile=subprocess.run(['cc','-std=c11','-Wall','-Wextra','-Werror','-I',str(ROOT/'native'),str(ROOT/'tests/native_m3_test.c'),'-o',str(output)],capture_output=True,text=True)
            self.assertEqual(compile.returncode,0,compile.stderr)
            result=subprocess.run([str(output)],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)

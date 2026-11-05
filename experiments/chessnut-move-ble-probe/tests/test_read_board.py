from pathlib import Path
import os,signal,subprocess,tempfile,time,unittest
ROOT=Path(__file__).resolve().parents[1]
class ReadBoardTests(unittest.TestCase):
 def setup_bundle(self,tmp):
  root=Path(tmp)/'probe';runtime=root/'runtime';runtime.mkdir(parents=True)
  (root/'.probe-owner').write_text('chessnut-ble-probe-v1\n')
  (root/'board-address').write_text('12:34:56:78:90:AB\n')
  (runtime/'build.json').write_text('{"source_sha256":"test-source"}\n')
  (runtime/'launch.sh').write_text('''#!/bin/sh
[ "$1" = read-once ] || exit 90
printf 'fake_peer=%s\\n' "$3"
printf 'connect_evidence api_status=-2\\n'
if [ "${TEST_SLEEP:-0}" = 1 ]; then exec sleep 30; fi
exit "${TEST_STATUS:-0}"
''')
  source=(ROOT/'scripts/read-board.sh').read_text().replace('/mnt/us/chessnut-ble-probe',str(root)).replace('/var/log/messages',str(root/'messages')).replace('/var/local/log/messages_',str(root/'archive_'))
  script=root/'read-board.sh';script.write_text(source)
  return root,script
 def test_pass_and_failure_are_printed_logged_and_returned_without_address_leak(self):
  for status in [0,37]:
   with tempfile.TemporaryDirectory() as tmp:
    root,script=self.setup_bundle(tmp)
    result=subprocess.run(['sh',str(script)],text=True,capture_output=True,env=dict(os.environ,TEST_STATUS=str(status)),timeout=5)
    self.assertEqual(result.returncode,status,result.stdout+result.stderr)
    self.assertIn('READ_BOARD_RESULT='+('PASS' if status==0 else 'FAIL'),result.stdout)
    self.assertIn('exit_status='+str(status),result.stdout)
    self.assertIn('connect_evidence',result.stdout)
    self.assertNotIn('12:34:56:78:90:AB',result.stdout)
    logs=list((root/'logs').glob('read-board-*.log'));self.assertEqual(len(logs),1)
    self.assertIn('READ_BOARD_RESULT=',logs[0].read_text())
    self.assertEqual(logs[0].stat().st_mode&0o777,0o600)
    self.assertEqual(list((root/'logs').glob('.read-board-*')),[])
 def test_check_does_not_start_a_session_and_missing_address_is_explicit(self):
  with tempfile.TemporaryDirectory() as tmp:
   root,script=self.setup_bundle(tmp)
   result=subprocess.run(['sh',str(script),'--check'],text=True,capture_output=True,timeout=5)
   self.assertEqual(result.returncode,0,result.stdout+result.stderr)
   self.assertIn('setup=PASS session=NOT_STARTED',result.stdout)
   self.assertNotIn('fake_peer',result.stdout)
   (root/'board-address').unlink()
   result=subprocess.run(['sh',str(script)],text=True,capture_output=True,timeout=5)
   self.assertNotEqual(result.returncode,0)
   self.assertIn('board-address',result.stdout+result.stderr)
 def test_termination_reaps_owned_runner_and_preserves_failure_report(self):
  with tempfile.TemporaryDirectory() as tmp:
   root,script=self.setup_bundle(tmp)
   with tempfile.TemporaryFile() as output:
    child=subprocess.Popen(['sh',str(script)],stdout=output,stderr=subprocess.STDOUT,env=dict(os.environ,TEST_SLEEP='1'))
    deadline=time.monotonic()+4
    while not list((root/'logs').glob('read-board-*.log')) and time.monotonic()<deadline:time.sleep(.02)
    time.sleep(.1);child.send_signal(signal.SIGTERM)
    self.assertEqual(child.wait(timeout=5),143)
    output.seek(0);text=output.read().decode()
    self.assertIn('READ_BOARD_RESULT=FAIL',text)
    self.assertEqual(list((root/'logs').glob('.read-board-*')),[])

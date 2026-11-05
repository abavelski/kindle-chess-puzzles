from pathlib import Path
import os,pty,subprocess,tempfile,unittest
ROOT=Path(__file__).resolve().parents[1]
class MoveBoardScriptTests(unittest.TestCase):
 def setup_bundle(self,tmp):
  root=Path(tmp)/'probe';runtime=root/'runtime';runtime.mkdir(parents=True)
  (root/'.probe-owner').write_text('chessnut-ble-probe-v1\n');(root/'board-address').write_text('12:34:56:78:90:ab\n')
  script=runtime/'move-board.sh';script.write_text((ROOT/'scripts/move-board.sh').read_text().replace('root=/mnt/us/chessnut-ble-probe',f'root={root}'))
  (runtime/'launch.sh').write_text('''#!/bin/sh
printf '%s\n' "$1" > "$PROBE_SESSION_LOG"
printf 'fake_guided_session=%s\n' "$1"
''')
  return root,script
 def test_preflight_and_nonterminal_never_connect(self):
  with tempfile.TemporaryDirectory() as tmp:
   root,script=self.setup_bundle(tmp)
   result=subprocess.run(['sh',str(script),'--check'],capture_output=True,text=True)
   self.assertEqual(result.returncode,0,result.stderr);self.assertIn('session=NOT_STARTED',result.stdout)
   result=subprocess.run(['sh',str(script)],capture_output=True,text=True)
   self.assertNotEqual(result.returncode,0);self.assertIn('interactive',result.stderr)
   self.assertFalse((root/'logs').exists())
 def test_guided_modes_use_private_logs_and_never_pipe_confirmation(self):
  for args,mode in [([], 'move-once'),(['--return'],'return-once')]:
   with tempfile.TemporaryDirectory() as tmp:
    root,script=self.setup_bundle(tmp);master,slave=pty.openpty()
    try:
     result=subprocess.run(['sh',str(script),*args],stdin=slave,stdout=slave,stderr=slave,timeout=5)
     self.assertEqual(result.returncode,0)
     screen=os.read(master,8192).decode()
     self.assertIn('e7 -> e2' if args else 'e2 -> e7',screen)
     logs=list((root/'logs').glob('*session.log'));self.assertEqual(len(logs),1)
     self.assertEqual(logs[0].read_text().strip(),mode)
     self.assertEqual(logs[0].stat().st_mode & 0o777,0o600)
    finally:os.close(master);os.close(slave)

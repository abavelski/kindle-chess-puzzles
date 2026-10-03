"""Exercise launcher ownership/cleanup using child processes, never services."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
LAUNCHER = ROOT / 'scripts/kindle_launch.sh'

class LauncherTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.env = dict(os.environ, KINDLE_CHESS_XREFRESH='', KINDLE_CHESS_LOCK=str(self.directory / 'lock'), KINDLE_CHESS_LOG=str(self.directory / 'log'))

    def child(self, body):
        path = self.directory / 'app'
        path.write_text('#!/bin/sh\n' + body + '\n')
        path.chmod(0o755)
        return path

    def launch(self, child):
        return subprocess.Popen(['sh', str(LAUNCHER), str(child)], env=self.env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def wait_for(self, path):
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            if path.exists():
                return
            time.sleep(.02)
        self.fail(f'timed out waiting for {path}')

    def test_normal_nonzero_and_repeated_launch_release_owned_lock(self):
        for code in [0, 7, 0, 0, 0]:
            proc = self.launch(self.child(f'exit {code}'))
            self.assertEqual(proc.wait(timeout=4), code)
            self.assertFalse((self.directory / 'lock').exists())

    def test_duplicate_launch_preserves_first_instances_lock(self):
        child = self.child('while :; do sleep 1; done')
        first = self.launch(child)
        try:
            self.wait_for(self.directory / 'lock/child.pid')
            second = self.launch(child)
            self.assertNotEqual(second.wait(timeout=4), 0)
            self.assertTrue((self.directory / 'lock').exists())
        finally:
            first.send_signal(signal.SIGTERM)
            first.wait(timeout=10)
        self.assertFalse((self.directory / 'lock').exists())

    def test_term_int_and_killed_child_cleanup(self):
        for mode in ['term', 'int', 'kill']:
            child = self.child('while :; do sleep 1; done')
            proc = self.launch(child)
            try:
                path = self.directory / 'lock/child.pid'
                self.wait_for(path)
                if mode == 'kill':
                    os.kill(int(path.read_text()), signal.SIGKILL)
                else:
                    proc.send_signal(signal.SIGTERM if mode == 'term' else signal.SIGINT)
                self.assertNotEqual(proc.wait(timeout=10), 0)
                self.assertFalse((self.directory / 'lock').exists())
            finally:
                if proc.poll() is None:
                    proc.kill()
                    proc.wait()

    def test_cleanup_escalates_a_child_that_ignores_term(self):
        ready = self.directory / 'ready'
        child = self.child(f"trap '' TERM\ntouch '{ready}'\nwhile :; do sleep 1; done")
        proc = self.launch(child)
        try:
            self.wait_for(ready)
            proc.send_signal(signal.SIGTERM)
            self.assertEqual(proc.wait(timeout=9), 143)
            self.assertFalse((self.directory / 'lock').exists())
            self.assertIn('child timeout', (self.directory / 'log').read_text())
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait()

    def test_exit_repaints_only_after_our_child_has_run(self):
        repaint = self.directory / 'repaint'
        invocation = self.directory / 'repaint-args'
        repaint.write_text(f"#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{invocation}'\n")
        repaint.chmod(0o755)
        timeout = self.directory / 'timeout'
        timeout.write_text("#!/bin/sh\n[ \"$1\" = '-k' ] && [ \"$2\" = '1' ] && [ \"$3\" = '5' ] || exit 9\nshift 3\nexec \"$@\"\n")
        timeout.chmod(0o755)
        self.env.update(KINDLE_CHESS_XREFRESH=str(repaint), KINDLE_CHESS_TIMEOUT=str(timeout))
        proc = self.launch(self.child('exit 7'))
        self.assertEqual(proc.wait(timeout=4), 7)
        self.assertEqual(invocation.read_text().strip(), '-display :0.0')
        invocation.unlink()
        # The Scribe default must repaint without an explicit hook override.
        default_repaint = self.directory / 'xrefresh'
        repaint.rename(default_repaint)
        self.env.pop('KINDLE_CHESS_XREFRESH')
        self.env['PATH'] = str(self.directory) + os.pathsep + os.environ['PATH']
        proc = self.launch(self.child('exit 0'))
        self.assertEqual(proc.wait(timeout=4), 0)
        self.assertEqual(invocation.read_text().strip(), '-display :0.0')
        invocation.unlink()
        proc = self.launch(self.directory / 'missing-app')
        self.assertNotEqual(proc.wait(timeout=4), 0)
        self.assertFalse(invocation.exists(), 'failed preflight must not repaint the stock UI')

    def test_static_contract(self):
        source = LAUNCHER.read_text()
        for fragment in ['trap cleanup EXIT', 'trap', 'INT', 'TERM', 'kill -TERM', 'kill -KILL']:
            self.assertIn(fragment, source)
        for forbidden in ['stop framework', 'stop lab126_gui', 'fbset ', 'EVIOCGRAB', 'wirelessEnable', 'governor']:
            self.assertNotIn(forbidden, source)
        subprocess.run(['sh', '-n', str(LAUNCHER)], check=True)

if __name__ == '__main__':
    unittest.main()

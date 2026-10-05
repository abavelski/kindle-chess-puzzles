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
        self.env = dict(os.environ, KINDLE_CHESS_DISPLAY_HANDOFF='0', KINDLE_CHESS_XREFRESH='', KINDLE_CHESS_LOCK=str(self.directory / 'lock'), KINDLE_CHESS_LOG=str(self.directory / 'log'))

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

    def wm_fixture(self, state='S'):
        proc_root = self.directory / 'proc'
        wm = proc_root / '123'
        wm.mkdir(parents=True)
        (wm / 'comm').write_text('awesome\n')
        (wm / 'stat').write_text('123 (awesome) ' + state + ' ' + '0 ' * 18 + '987\n')
        (wm / 'status').write_text('State:\t' + state + ' (test)\n')
        x = proc_root / '456'
        x.mkdir()
        (x / 'comm').write_text('Xorg\n')
        (x / 'stat').write_text('456 (Xorg) S ' + '0 ' * 18 + '654\n')
        (x / 'status').write_text('State:\tS (test)\n')
        pidof = self.directory / 'pidof'
        pidof.write_text('#!/bin/sh\ncase "$1" in awesome) echo 123;; Xorg) echo 456;; *) exit 1;; esac\n')
        pidof.chmod(0o755)
        signals = self.directory / 'signals'
        control = self.directory / 'signal'
        control.write_text(f'#!/bin/sh\necho "$*" >> "{signals}"\n')
        control.chmod(0o755)
        self.env.update(KINDLE_CHESS_DISPLAY_HANDOFF='1',
                        KINDLE_CHESS_PROC_ROOT=str(proc_root),
                        KINDLE_CHESS_SIGNAL=str(control),
                        PATH=str(self.directory) + os.pathsep + os.environ['PATH'])
        return wm, signals

    def test_display_handoff_precedes_child_and_restores_on_exit_and_term(self):
        _, signals = self.wm_fixture()
        for mode in ['normal', 'term', 'kill']:
            signals.unlink(missing_ok=True)
            ready = self.directory / 'ready'
            ready.unlink(missing_ok=True)
            body = f'cat "{signals}" > "{ready}"\n'
            body += 'exit 7' if mode == 'normal' else 'exec sleep 30'
            proc = self.launch(self.child(body))
            try:
                self.wait_for(ready)
                self.assertEqual(ready.read_text().splitlines(), ['-STOP 123', '-STOP 456'])
                if mode == 'term':
                    proc.terminate()
                elif mode == 'kill':
                    self.wait_for(self.directory / 'lock/child.pid')
                    os.kill(int((self.directory / 'lock/child.pid').read_text()), signal.SIGKILL)
                proc.wait(timeout=10)
                self.assertEqual(signals.read_text().splitlines(), ['-STOP 123', '-STOP 456', '-CONT 456', '-CONT 123'])
            finally:
                if proc.poll() is None:
                    proc.terminate()
                    proc.wait(timeout=10)

    def test_display_handoff_refuses_already_stopped_wm(self):
        _, signals = self.wm_fixture('T')
        ready = self.directory / 'ready'
        proc = self.launch(self.child(f'touch "{ready}"'))
        self.assertNotEqual(proc.wait(timeout=4), 0)
        self.assertFalse(ready.exists())
        self.assertFalse(signals.exists())

    def test_display_cleanup_does_not_signal_reused_wm_pid(self):
        wm, signals = self.wm_fixture()
        ready = self.directory / 'ready'
        proc = self.launch(self.child(f'touch "{ready}"\nexec sleep 30'))
        try:
            self.wait_for(ready)
            (wm / 'stat').write_text((wm / 'stat').read_text().replace('987', '988'))
            proc.terminate()
            proc.wait(timeout=10)
            self.assertEqual(signals.read_text().splitlines(), ['-STOP 123', '-STOP 456', '-CONT 456'])
            self.assertIn('identity changed', (self.directory / 'log').read_text())
        finally:
            if proc.poll() is None:
                proc.terminate()
                proc.wait(timeout=10)

    def test_display_handoff_validates_xorg_before_pausing_either_process(self):
        wm, signals = self.wm_fixture()
        (wm.parent / '456/status').write_text('State:\tT (stopped)\n')
        ready = self.directory / 'ready'
        proc = self.launch(self.child(f'touch "{ready}"'))
        self.assertNotEqual(proc.wait(timeout=4), 0)
        self.assertFalse(ready.exists())
        self.assertFalse(signals.exists())

    def test_wake_handoff_repeats_and_refuses_reused_or_dead_processes(self):
        wm, signals = self.wm_fixture()
        app = wm.parent / str(os.getpid())
        app.mkdir()
        (app / 'comm').write_text('kindle-chess\n')
        (app / 'stat').write_text(f'{os.getpid()} (kindle-chess) S ' + '0 ' * 18 + '789\n')
        (app / 'status').write_text('State:\tS (test)\n')
        self.env.update(KINDLE_CHESS_WAKE_XORG_PID='456', KINDLE_CHESS_WAKE_XORG_START='654')
        def invoke(mode):
            return subprocess.run(['sh', str(LAUNCHER), mode, str(os.getpid())], env=self.env, capture_output=True, timeout=5)
        for _ in range(3):
            signals.unlink(missing_ok=True)
            self.assertEqual(invoke('--wake-display').returncode, 0)
            self.assertEqual(signals.read_text().splitlines(), ['-CONT 456'])
            self.assertEqual(invoke('--finish-wake-display').returncode, 0)
            self.assertEqual(signals.read_text().splitlines(), ['-CONT 456', '-STOP 456'])
        signals.unlink()
        (wm.parent / '456/stat').write_text('456 (Xorg) S ' + '0 ' * 18 + '655\n')
        self.assertNotEqual(invoke('--wake-display').returncode, 0)
        self.assertFalse(signals.exists())
        (wm.parent / '456/stat').write_text('456 (Xorg) S ' + '0 ' * 18 + '654\n')
        self.assertEqual(invoke('--wake-display').returncode, 0)
        (app / 'stat').unlink()
        self.assertNotEqual(invoke('--finish-wake-display').returncode, 0)
        self.assertEqual(signals.read_text().splitlines(), ['-CONT 456'])

    def test_supervisor_exports_wake_identity_only_for_owned_display(self):
        capture = self.directory / 'wake-env'
        body = f'printf "%s\\n" "${{KINDLE_CHESS_WAKE_HOOK-unset}}" "${{KINDLE_CHESS_WAKE_XORG_PID-unset}}" "${{KINDLE_CHESS_WAKE_XORG_START-unset}}" > "{capture}"'
        self.env['KINDLE_CHESS_WAKE_HOOK'] = '/stale/hook'
        proc = self.launch(self.child(body))
        self.assertEqual(proc.wait(timeout=4), 0)
        self.assertEqual(capture.read_text().splitlines(), ['unset', 'unset', 'unset'])
        self.wm_fixture()
        proc = self.launch(self.child(body))
        self.assertEqual(proc.wait(timeout=4), 0)
        self.assertEqual(capture.read_text().splitlines(), [str(LAUNCHER), '456', '654'])

    def test_static_contract(self):
        source = LAUNCHER.read_text()
        for fragment in ['trap cleanup EXIT', 'trap', 'INT', 'TERM', 'kill -TERM', 'kill -KILL']:
            self.assertIn(fragment, source)
        for forbidden in ['stop framework', 'stop lab126_gui', 'fbset ', 'EVIOCGRAB', 'wirelessEnable', 'governor']:
            self.assertNotIn(forbidden, source)
        subprocess.run(['sh', '-n', str(LAUNCHER)], check=True)

if __name__ == '__main__':
    unittest.main()

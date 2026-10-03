"""Mock the narrow input ioctl boundary; never grab a host input device."""
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class InputBridgeTests(unittest.TestCase):
    def test_foreground_startup_requires_grab_without_an_opt_in_flag(self):
        source = (ROOT / 'app/kindle-chess/src/main.rs').read_text()
        self.assertNotIn('--exclusive-input', source)
        self.assertNotIn('if exclusive_input', source)
        self.assertLess(source.index('input.take_exclusive()?'), source.index('display.present_damage('))

    def test_owned_grab_release_and_errno(self):
        source = r'''
#include <assert.h>
#include <stdarg.h>
#include <errno.h>
#define ioctl mock_ioctl
#include "input_bridge.c"
static int calls, failure;
static unsigned long expected;
int mock_ioctl(int fd, unsigned long request, ...) {
    assert(fd==7 && request==EVIOCGRAB);
    va_list args; va_start(args, request);
    assert(va_arg(args, unsigned long)==expected);
    va_end(args); calls++;
    if (failure) { errno=EBUSY; return -1; }
    return 0;
}
int main(void) {
    expected=1;
    assert(kcp_input_grab(7,1)==0 && calls==1);
    expected=0;
    assert(kcp_input_grab(7,0)==0 && calls==2);
    expected=1; failure=1;
    assert(kcp_input_grab(7,1)==-EBUSY && calls==3);
    return 0;
}
'''
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)
            (path/'linux').mkdir()
            (path/'linux/input.h').write_text('#define EVIOCGRAB 0x40044590UL\n')
            (path/'test.c').write_text(source)
            subprocess.run(['cc','-std=gnu11','-Wall','-Wextra','-Werror','-I',str(path),'-I',str(ROOT/'crates/fbink-sys/c'),str(path/'test.c'),'-o',str(path/'test')],check=True)
            subprocess.run([str(path/'test')],check=True)

if __name__=='__main__': unittest.main()

"""Compile the actual C bridge against the pinned header and mocked FBInk calls."""
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]

class BridgeTests(unittest.TestCase):
    def test_region_mode_mapping_wait_and_unsupported_fallback(self):
        source = r'''
#include <assert.h>
#include <errno.h>
#include "fbink_bridge.c"
static int calls, failure, waits;
static unsigned char mode;
int fbink_open(void) { return 4; }
int fbink_close(int fd) { assert(fd == 4); return 0; }
int fbink_init(int fd, const FBInkConfig* cfg) { (void)fd; (void)cfg; return 0; }
int fbink_reinit(int fd, const FBInkConfig* cfg) { (void)fd; (void)cfg; return 0; }
FBINK_TARGET_T fbink_target(void) { return FBINK_TARGET_KINDLE; }
const char* fbink_version(void) { return "mock"; }
void fbink_get_state(const FBInkConfig* cfg, FBInkState* state) { (void)cfg; state->screen_width=1860; state->screen_height=2480; }
int fbink_print_raw_data(int fd, const unsigned char* data, int w, int h, size_t len, short x, short y, const FBInkConfig* cfg) {
    (void)data; assert(fd==4 && w==2 && h==3 && len==6 && x==20 && y==30);
    assert(cfg->ignore_alpha && !cfg->no_refresh);
    assert(cfg->is_flashing == (mode==0 || mode==4));
    calls++;
    if (calls==1) {
        assert(cfg->wfm_mode == (mode==2 ? WFM_GC16 : mode==3 ? WFM_DU : WFM_AUTO));
        if (failure==1) { errno=EINVAL; return -1; } // pinned driver returns EXIT_FAILURE
        if (failure==2) { errno=EIO; return -1; }
        if (failure==3) return -ENOSYS;
    } else { assert(cfg->wfm_mode==WFM_AUTO); }
    return 0;
}
int fbink_wait_for_complete(int fd, uint32_t marker) { assert(fd==4 && marker==LAST_MARKER); waits++; return 0; }
int main(void) {
    unsigned char pixels[6]={0};
    for (mode=0; mode<5; mode++) {
        failure=0; calls=0;
        assert(kcp_fbink_present_region(4,pixels,2,3,6,20,30,mode)==0);
        assert(calls==1 && waits==0);
    }
    mode=3; calls=0; failure=1;
    assert(kcp_fbink_present_region(4,pixels,2,3,6,20,30,mode)==0 && calls==2);
    mode=2; calls=0; failure=3;
    assert(kcp_fbink_present_region(4,pixels,2,3,6,20,30,mode)==0 && calls==2);
    calls=0; failure=2;
    assert(kcp_fbink_present_region(4,pixels,2,3,6,20,30,mode)<0 && calls==1);
    assert(kcp_fbink_wait(4)==0 && waits==1);
    return 0;
}
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / 'contract.c').write_text(source)
            # Only opaque framebuffer pointer declarations occur in this header.
            (path / 'linux').mkdir()
            (path / 'linux/fb.h').write_text('struct fb_var_screeninfo; struct fb_fix_screeninfo;\n')
            subprocess.run(['cc', '-std=gnu11', '-Wall', '-Wextra', '-Werror', '-DFBINK_FOR_KINDLE', '-I', str(path), '-I', str(ROOT / 'vendor/FBInk'), '-I', str(ROOT / 'crates/fbink-sys/c'), str(path / 'contract.c'), '-o', str(path / 'contract')], check=True)
            subprocess.run([str(path / 'contract')], check=True)

if __name__ == '__main__':
    unittest.main()

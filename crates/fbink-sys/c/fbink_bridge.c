#include <errno.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "fbink.h"

typedef struct {
    uint32_t width;
    uint32_t height;
    uint32_t scanline_stride;
    uint32_t bpp;
    uint8_t rotation;
    uint8_t is_y8;
} KcpFbInkState;

int
kcp_fbink_open(void)
{
    const int fbfd = fbink_open();
    if (fbfd < 0) {
        return fbfd;
    }

    FBInkConfig cfg = { 0 };
    cfg.is_quiet = true;
    const int rv = fbink_init(fbfd, &cfg);
    if (rv < 0) {
        (void) fbink_close(fbfd);
        return rv;
    }

    if (fbink_target() != FBINK_TARGET_KINDLE) {
        (void) fbink_close(fbfd);
        return -ENODEV;
    }

    return fbfd;
}

int
kcp_fbink_reinit(int fbfd)
{
    FBInkConfig cfg = { 0 };
    cfg.is_quiet = true;
    const int rv = fbink_reinit(fbfd, &cfg);
    return rv < 0 ? rv : 0;
}

int
kcp_fbink_get_state(KcpFbInkState* out)
{
    if (out == NULL) {
        return -EINVAL;
    }

    FBInkConfig cfg = { 0 };
    FBInkState state = { 0 };
    fbink_get_state(&cfg, &state);

    out->width = state.screen_width;
    out->height = state.screen_height;
    out->scanline_stride = state.scanline_stride;
    out->bpp = state.bpp;
    out->rotation = state.current_rota;
    out->is_y8 = state.pixel_format == FBINK_PXFMT_Y8 ? 1U : 0U;
    return 0;
}

int
kcp_fbink_present_gray8(
    int fbfd,
    const uint8_t* data,
    uint32_t width,
    uint32_t height,
    size_t len)
{
    if (data == NULL || width == 0U || height == 0U) {
        return -EINVAL;
    }

    FBInkConfig cfg = { 0 };
    cfg.ignore_alpha = true;
    cfg.is_flashing = true;

    const int rv = fbink_print_raw_data(
        fbfd,
        data,
        (int) width,
        (int) height,
        len,
        0,
        0,
        &cfg);
    if (rv < 0) {
        return rv;
    }

    const int wait_rv = fbink_wait_for_complete(fbfd, LAST_MARKER);
    return wait_rv < 0 ? wait_rv : 0;
}

int
kcp_fbink_close(int fbfd)
{
    return fbink_close(fbfd);
}

const char*
kcp_fbink_version(void)
{
    return fbink_version();
}

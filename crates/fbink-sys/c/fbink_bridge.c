#include <errno.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

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
kcp_fbink_present_region(
    int fbfd,
    const uint8_t* data,
    uint32_t width,
    uint32_t height,
    size_t len,
    uint32_t left,
    uint32_t top,
    uint8_t mode)
{
    if (data == NULL || width == 0U || height == 0U) {
        return -EINVAL;
    }

    FBInkConfig cfg = { 0 };
    cfg.ignore_alpha = true;
    cfg.is_flashing = mode == 0U || mode == 4U;
    switch (mode) {
        case 0U: case 1U: case 4U: cfg.wfm_mode = WFM_AUTO; break;
        case 2U: cfg.wfm_mode = WFM_GC16; break;
        case 3U: cfg.wfm_mode = WFM_DU; break;
        default: return -EINVAL;
    }

    errno = 0;
    int rv = fbink_print_raw_data(
        fbfd,
        data,
        (int) width,
        (int) height,
        len,
        (int) left,
        (int) top,
        &cfg);
    const int submit_errno = errno;
    // Retry unsupported explicit modes using the baseline AUTO waveform.
    // Do not hide unrelated I/O or submission failures.
    if ((mode == 2U || mode == 3U) &&
        (rv == -EINVAL || rv == -ENOSYS || rv == -EOPNOTSUPP ||
         (rv == -1 && (submit_errno == EINVAL || submit_errno == ENOSYS || submit_errno == EOPNOTSUPP)))) {
        fprintf(stderr, "FBInk: mode %u unsupported; retrying AUTO\n", mode);
        cfg.wfm_mode = WFM_AUTO;
        rv = fbink_print_raw_data(fbfd, data, (int) width, (int) height,
                                  len, (int) left, (int) top, &cfg);
    }
    return rv < 0 ? rv : 0;
}

int
kcp_fbink_wait(int fbfd)
{
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

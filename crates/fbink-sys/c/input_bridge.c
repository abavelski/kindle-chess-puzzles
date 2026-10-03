#include <errno.h>
#include <stdint.h>
#include <sys/ioctl.h>
#include <linux/input.h>

// The caller owns this evdev descriptor. Never discover or steal another grab.
int kcp_input_grab(int fd, uint8_t enabled)
{
    const int rv = ioctl(fd, EVIOCGRAB, (unsigned long) (enabled != 0U));
    return rv < 0 ? -errno : 0;
}

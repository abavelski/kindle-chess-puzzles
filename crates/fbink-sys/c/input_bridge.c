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

#include <poll.h>

int kcp_input_wait(int first, int second)
{
    struct pollfd descriptors[2] = {
        { .fd = first, .events = POLLIN },
        { .fd = second, .events = POLLIN }
    };
    int rv;
    do {
        rv = poll(descriptors, 2, -1);
    } while (rv < 0 && errno == EINTR);
    if (rv < 0) return -errno;
    for (int i = 0; i < 2; ++i) {
        if (descriptors[i].revents & (POLLERR | POLLHUP | POLLNVAL)) return -EIO;
    }
    for (int i = 0; i < 2; ++i) {
        if (descriptors[i].revents & POLLIN) return i;
    }
    return -EIO;
}

// Power pipe EOF is readable so the owner can reap/restart the finite listener.
int kcp_input_wait_power_timeout(int first, int second, int power, int timeout_ms)
{
    struct pollfd descriptors[3] = {
        { .fd = first, .events = POLLIN },
        { .fd = second, .events = POLLIN },
        { .fd = power, .events = POLLIN }
    };
    int rv;
    do { rv = poll(descriptors, 3, timeout_ms); } while (rv < 0 && errno == EINTR);
    if (rv < 0) return -errno;
    if (rv == 0) return 3;
    if (descriptors[2].revents & (POLLIN | POLLHUP)) return 2;
    for (int i = 0; i < 3; ++i) {
        if (descriptors[i].revents & (POLLERR | POLLHUP | POLLNVAL)) return -EIO;
    }
    for (int i = 0; i < 2; ++i) {
        if (descriptors[i].revents & POLLIN) return i;
    }
    return -EIO;
}

// Caller retries EINTR against its own monotonic deadline.
int kcp_pipe_wait(int fd, int timeout_ms)
{
    struct pollfd descriptor = { .fd = fd, .events = POLLIN };
    int rv = poll(&descriptor, 1, timeout_ms);
    if (rv < 0) return -errno;
    if (rv == 0) return 0;
    if (descriptor.revents & (POLLIN | POLLHUP)) return 1;
    return -EIO;
}

int kcp_input_wait_power(int first, int second, int power) {
    return kcp_input_wait_power_timeout(first, second, power, -1);
}

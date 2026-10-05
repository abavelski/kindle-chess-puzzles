//! Narrow owned-file boundary for exclusive Linux evdev input.
use std::{fs::File, io};

pub struct ExclusiveInput {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    file: File,
}

impl ExclusiveInput {
    /// Take ownership of a descriptor sharing the application's input stream.
    /// A failed acquisition closes it; process death closes all owned handles.
    pub fn acquire(file: File) -> Result<Self, io::Error> {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            use std::os::fd::AsRawFd;
            let code = unsafe { kcp_input_grab(file.as_raw_fd(), 1) };
            if code < 0 {
                return Err(io::Error::from_raw_os_error(-code));
            }
            Ok(Self { file })
        }
        #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
        {
            drop(file);
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "exclusive evdev input is only available on the Kindle target",
            ))
        }
    }
}

impl Drop for ExclusiveInput {
    fn drop(&mut self) {
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        {
            use std::os::fd::AsRawFd;
            let code = unsafe { kcp_input_grab(self.file.as_raw_fd(), 0) };
            if code < 0 {
                eprintln!(
                    "exclusive input release: {}",
                    io::Error::from_raw_os_error(-code)
                );
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
extern "C" {
    fn kcp_input_grab(fd: std::os::raw::c_int, enabled: u8) -> std::os::raw::c_int;
}

/// Wait for one of two owned input files, returning its index.
pub fn wait_input(first: &File, second: &File) -> Result<usize, io::Error> {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    {
        use std::os::fd::AsRawFd;
        let code = unsafe { kcp_input_wait(first.as_raw_fd(), second.as_raw_fd()) };
        if code < 0 {
            Err(io::Error::from_raw_os_error(-code))
        } else {
            Ok(code as usize)
        }
    }
    #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
    {
        let _ = (first, second);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "evdev polling requires the Kindle target",
        ))
    }
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
extern "C" {
    fn kcp_input_wait(
        first: std::os::raw::c_int,
        second: std::os::raw::c_int,
    ) -> std::os::raw::c_int;
}

/// Multiplex owned finger, pen and passive power-event descriptors.
pub fn wait_input_power(
    first: &File,
    second: &File,
    power: &impl std::os::fd::AsRawFd,
) -> Result<usize, io::Error> {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    {
        use std::os::fd::AsRawFd;
        let code = unsafe {
            kcp_input_wait_power(first.as_raw_fd(), second.as_raw_fd(), power.as_raw_fd())
        };
        if code < 0 {
            Err(io::Error::from_raw_os_error(-code))
        } else {
            Ok(code as usize)
        }
    }
    #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
    {
        let _ = (first, second, power);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "evdev polling requires the Kindle target",
        ))
    }
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
extern "C" {
    fn kcp_input_wait_power(
        first: std::os::raw::c_int,
        second: std::os::raw::c_int,
        power: std::os::raw::c_int,
    ) -> std::os::raw::c_int;
}

/// Wait on an owned pipe with a finite deadline; EOF remains readable.
pub fn wait_pipe(pipe: &impl std::os::fd::AsRawFd, timeout_ms: i32) -> io::Result<bool> {
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    {
        let code = unsafe { kcp_pipe_wait(pipe.as_raw_fd(), timeout_ms) };
        if code < 0 {
            Err(io::Error::from_raw_os_error(-code))
        } else {
            Ok(code != 0)
        }
    }
    #[cfg(not(all(target_os = "linux", target_arch = "arm")))]
    {
        let _ = (pipe, timeout_ms);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "power pipe polling requires the Kindle target",
        ))
    }
}

#[cfg(all(target_os = "linux", target_arch = "arm"))]
extern "C" {
    fn kcp_pipe_wait(
        fd: std::os::raw::c_int,
        timeout_ms: std::os::raw::c_int,
    ) -> std::os::raw::c_int;
}

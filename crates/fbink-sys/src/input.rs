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

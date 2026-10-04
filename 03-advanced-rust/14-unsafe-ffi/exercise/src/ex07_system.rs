//! Exercise 7: System libraries -- POSIX calls, Unix only.

#[cfg(unix)]
mod sys {
    use std::ffi::{c_char, c_int};
    unsafe extern "C" {
        pub safe fn getpid() -> c_int;
        pub fn gethostname(name: *mut c_char, len: usize) -> c_int;
    }
}

#[cfg(unix)]
pub fn process_id() -> u32 {
    todo!("Exercise 7")
}

/// POSIX reports errors as -1 plus `errno`; `io::Error::last_os_error()`
/// reads errno and turns it into a proper Rust error with a message.
#[cfg(unix)]
pub fn hostname() -> std::io::Result<String> {
    todo!("Exercise 7")
}

pub fn run() {
    todo!("Exercise 7")
}

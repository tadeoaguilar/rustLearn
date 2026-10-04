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
    sys::getpid() as u32
}

/// POSIX reports errors as -1 plus `errno`; `io::Error::last_os_error()`
/// reads errno and turns it into a proper Rust error with a message.
#[cfg(unix)]
pub fn hostname() -> std::io::Result<String> {
    let mut buf = vec![0u8; 256];
    // SAFETY: buf is writable for buf.len() bytes.
    let rc = unsafe { sys::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // Not guaranteed NUL-terminated if truncated: stop at the first NUL or the end.
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    buf.truncate(end);
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn run() {
    #[cfg(unix)]
    {
        println!(
            "getpid()      = {}  (std::process::id() = {})",
            process_id(),
            std::process::id()
        );
        println!("gethostname() = {:?}", hostname());
    }
    #[cfg(not(unix))]
    println!("Exercise 7 uses POSIX functions; it's compiled only on Unix.");
}

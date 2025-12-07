
use crate::{apperr, helpers::GIBI};
use std::{ffi::c_int, mem::ManuallyDrop};
use stdext::arena::{Arena, ArenaString};

struct State {
    stdin: libc::c_int,
    stdin_flags: libc::c_int,
    stdout: libc::c_int,
    stdout_initial_termios: Option<libc::termios>,
    inject_resize: bool,
    // Buffer for incomplete UTF-8 sequences (max 4 bytes needed)
    utf8_buf: [u8; 4],
    utf8_len: usize,
}

static mut STATE: State = State {
    stdin: libc::STDIN_FILENO,
    stdin_flags: 0,
    stdout: libc::STDOUT_FILENO,
    stdout_initial_termios: None,
    inject_resize: false,
    utf8_buf: [0; 4],
    utf8_len: 0,
};

pub struct Deinit;

impl Drop for Deinit {
    fn drop(&mut self) {
        
    }
}

pub fn init() -> Deinit {
    Deinit
}

pub fn write_stdout(text: &str) {
    if text.is_empty() {
        return;
    }

    // tty를 blocking으로 변경
    set_tty_nonblocking(false);

    let buf = text.as_bytes();
    let mut written = 0;

    while written < buf.len() {
        let w = &buf[written..];
        let w = &buf[..w.len().min(GIBI)];
        let n = unsafe { libc::write(STATE.stdout, w.as_ptr() as *const _, w.len()) };

        if n >= 0 {
            written += n as usize;
            continue;
        }

        let err = errno();
        if err != libc::EINTR {
            return;
        }
    }
}

pub fn switch_modes() -> apperr::Result<()> {
    unsafe {
        Ok(())
    }
}

pub fn apperr_format(f: &mut std::fmt::Formatter<'_>, code: u32) -> std::fmt::Result {
    write!(f, "Error {code}")?;
    
    Ok(())
}

pub fn preferred_languages(arena: &Arena) -> Vec<ArenaString<'_>, &Arena> {
    let mut locales = Vec::new_in(arena);

    for key in ["LANGUAGE", "LC_ALL", "LANG"] {
        if let Ok(val) = std::env::var(key) && !val.is_empty() {
            locales.extend(val.split(':').filter(|s| !s.is_empty()).map(|s| {
                // Replace all underscores with dashes,
                // because the localization code expects pt-br, not pt_BR.
                let mut res = Vec::new_in(arena);
                res.extend(s.as_bytes().iter().map(|&b| if b == b'_' { b'-' } else { b }));
                unsafe { ArenaString::from_utf8_unchecked(res) }
            }));
            break;
        }
    }

    locales
}

/// 터미널의 file descriptor를 non-blocking 또는 blocking 모드로 전환
fn set_tty_nonblocking(nonblock: bool) {
    unsafe {
        let is_nonblock = (STATE.stdin_flags & libc::O_NONBLOCK) != 0;
        if is_nonblock != nonblock {
            // libc::O_NONBLOCK 비트의 값 토글
            STATE.stdin_flags ^= libc::O_NONBLOCK;
            let _ = libc::fcntl(STATE.stdin, libc::F_SETFL, STATE.stdin_flags);
        }
    }
}

#[inline]
fn errno() -> i32 {
    // Under `-O -Copt-level=s` the 1.87 compiler fails to fully inline and
    // remove the raw_os_error() call. This leaves us with the drop() call.
    // ManuallyDrop fixes that and results in a direct `std::sys::os::errno` call.
    ManuallyDrop::new(std::io::Error::last_os_error()).raw_os_error().unwrap_or(0)
}

#[cold]
pub fn get_last_error() -> apperr::Error {
    errno_to_apperr(errno())
}

#[inline]
pub(crate) fn io_error_to_apperr(err: std::io::Error) -> apperr::Error {
    errno_to_apperr(err.raw_os_error().unwrap_or(0))
}

const fn errno_to_apperr(no: c_int) -> apperr::Error {
    apperr::Error::new_sys(if no < 0 { 0 } else { no as u32 })
}

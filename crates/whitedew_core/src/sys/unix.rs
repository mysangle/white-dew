
use crate::{
    apperr,
    helpers::{GIBI, KIBI},
};
use std::{
    ffi::c_int,
    fs::File,
    mem::{self, ManuallyDrop, MaybeUninit},
    os::fd::FromRawFd as _,
    ptr::null_mut,
    thread,
    time,
};
use stdext::{
    arena::{Arena, ArenaString, scratch_arena},
    arena_format,
};

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
        unsafe {
            #[allow(static_mut_refs)]
            if let Some(termios) = STATE.stdout_initial_termios.take() {
                // Restore the original terminal modes.
                // 터미널을 원래 상태로 복원
                libc::tcsetattr(STATE.stdout, libc::TCSANOW, &termios);
            }
        }
    }
}

pub fn init() -> Deinit {
    Deinit
}

pub fn inject_window_size_into_stdin() {
    unsafe {
        STATE.inject_resize = true;
    }
}

extern "C" fn sigwinch_handler(_: libc::c_int) {
    unsafe {
        STATE.inject_resize = true;
    }
}

/// stdout에 text 쓰기
pub fn write_stdout(text: &str) {
    if text.is_empty() {
        return;
    }

    // If we don't set the TTY to blocking mode,
    // the write will potentially fail with EAGAIN.
    //
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

/// Reads from stdin.
///
/// Returns `None` if there was an error reading from stdin.
/// Returns `Some("")` if the given timeout was reached.
/// Otherwise, it returns the read, non-empty string.
pub fn read_stdin(arena: &Arena, mut timeout: time::Duration) -> Option<ArenaString<'_>> {
    unsafe {
        if STATE.inject_resize {
            timeout = time::Duration::ZERO;
        }

        let read_poll = timeout != time::Duration::MAX;
        let mut buf = Vec::new_in(arena);

        // We don't know if the input is valid UTF8, so we first use a Vec and then
        // later turn it into UTF8 using `from_utf8_lossy_owned`.
        // It is important that we allocate the buffer with an explicit capacity,
        // because we later use `spare_capacity_mut` to access it.
        buf.reserve(4 * KIBI);

        // We got some leftover broken UTF8 from a previous read? Prepend it.
        // 이전 read_stdin 호출에서 읽고 남은 값이 있으면 이를 버퍼의 앞에 넣는다.
        if STATE.utf8_len != 0 {
            buf.extend_from_slice(&STATE.utf8_buf[..STATE.utf8_len]);
            STATE.utf8_len = 0;
        }

        // 입력 대기 및 읽기 루프
        loop {
            if timeout != time::Duration::MAX {
                let beg = time::Instant::now();

                let mut pollfd = libc::pollfd { fd: STATE.stdin, events: libc::POLLIN, revents: 0 };
                let ret;
                // #[cfg(target_os = "linux")]
                // {
                //     let ts = libc::timespec {
                //         tv_sec: timeout.as_secs() as libc::time_t,
                //         tv_nsec: timeout.subsec_nanos() as libc::c_long,
                //     };
                //     ret = libc::ppoll(&mut pollfd, 1, &ts, ptr::null());
                // }
                // #[cfg(not(target_os = "linux"))]
                {
                    // 입력을 기다리기
                    ret = libc::poll(&mut pollfd, 1, timeout.as_millis() as libc::c_int);
                }
                if ret < 0 {
                    return None; // Error? Let's assume it's an EOF.
                }
                if ret == 0 {
                    break; // Timeout? We can stop reading.
                }

                timeout = timeout.saturating_sub(beg.elapsed());
            }

            // If we're asked for a non-blocking read we need
            // to manipulate `O_NONBLOCK` and vice versa.
            set_tty_nonblocking(read_poll);

            // Read from stdin.
            let spare = buf.spare_capacity_mut();
            // 입력 읽기
            let ret = libc::read(STATE.stdin, spare.as_mut_ptr() as *mut _, spare.len());
            if ret > 0 {
                buf.set_len(buf.len() + ret as usize);
                break;
            }
            if ret == 0 {
                return None; // EOF
            }
            if ret < 0 {
                match errno() {
                    libc::EINTR if STATE.inject_resize => break,
                    libc::EAGAIN if timeout == time::Duration::ZERO => break,
                    libc::EINTR | libc::EAGAIN => {}
                    _ => return None,
                }
            }
        }

        // 후처리
        if !buf.is_empty() {
            // 읽어온 데이터(buf)의 맨 끝에 잘린 멀티바이트 UTF-8 문자가 있는지 검사

            // We only need to check the last 3 bytes for UTF-8 continuation bytes,
            // because we should be able to assume that any 4 byte sequence is complete.
            let lim = buf.len().saturating_sub(3);
            let mut off = buf.len() - 1;

            // UTF-8:
            // 바이트 종류 비트 패턴
            // ASCII    0xxxxxxx
            // 리드 바이트 110xxxxx, 1110xxxx, 11110xxx
            // 연속 바이트 10xxxxxx
            //
            // Find the start of the last potentially incomplete UTF-8 sequence.
            // 
            // UTF-8 시퀀스의 시작 후보(리드 바이트)를 찾는다(연속 바이트이면 건너뛴다.)
            while off > lim && buf[off] & 0b1100_0000 == 0b1000_0000 {
                off -= 1;
            }

            let seq_len = match buf[off] {
                b if b & 0b1000_0000 == 0 => 1,
                b if b & 0b1110_0000 == 0b1100_0000 => 2,
                b if b & 0b1111_0000 == 0b1110_0000 => 3,
                b if b & 0b1111_1000 == 0b1111_0000 => 4,
                // If the lead byte we found isn't actually one, we don't cache it.
                // `from_utf8_lossy_owned` will replace it with U+FFFD.
                // 문제가 있는 정상적이지 않은 u8
                _ => 0,
            };

            // Cache incomplete sequence if any.
            if off + seq_len > buf.len() {
                STATE.utf8_len = buf.len() - off;
                STATE.utf8_buf[..STATE.utf8_len].copy_from_slice(&buf[off..]);
                buf.truncate(off);
            }
        }

        let mut result = ArenaString::from_utf8_lossy_owned(buf);

        // We received a SIGWINCH? Add a fake window size sequence for our input parser.
        // I prepend it so that on startup, the TUI system gets first initialized with a size.
        if STATE.inject_resize {
            STATE.inject_resize = false;
            // 새로운 터미널의 크기를 가져온다.
            let (w, h) = get_window_size();
            if w > 0 && h > 0 {
                let scratch = scratch_arena(Some(arena));
                // "\x1b[8;{h};{w}t": 윈도우(터미널) 크기를 변경하라는 요청
                // 최종 결과 문자열의 맨 앞에 삽입하여 사용자가 키보드로 창 크기 정보를 입력한 것처럼 이 이벤트를 처리하도록 한다.
                let seq = arena_format!(&scratch, "\x1b[8;{h};{w}t");
                result.replace_range(0..0, &seq);
            }
        }

        result.shrink_to_fit();
        Some(result)
    }
}

pub fn switch_modes() -> apperr::Result<()> {
    unsafe {
        // Reopen stdin if it's redirected (= piped input).
        // 표준 입력 재설정
        if libc::isatty(STATE.stdin) == 0 {
            // stdin이 tty가 아니면 redirect된 것이므로 stdin을 open 한다.
            // 사용자 입력은 터미널로 받아야 하므로 /dev/tty를 읽기 모드로 연다.
            STATE.stdin = check_int_return(libc::open(c"/dev/tty".as_ptr(), libc::O_RDONLY))?;
        }
        // Store the stdin flags so we can more easily toggle `O_NONBLOCK` later on.
        // stdin의 현재 파일 디스크립터 플래그를 저장. 나중에 non-blocking 모드를 쉽게 켜고끄는데 사용한다.
        STATE.stdin_flags = check_int_return(libc::fcntl(STATE.stdin, libc::F_GETFL))?;

        // Set STATE.inject_resize to true whenever we get a SIGWINCH.
        // SIGWINCH: 터미널 창 크기가 변경될 떄 운영체제가 프로세스에 보내는 시그널
        // SIGWINCH 시그널 핸들러 등록
        let mut sigwinch_action: libc::sigaction = mem::zeroed();
        sigwinch_action.sa_sigaction = sigwinch_handler as *const () as libc::sighandler_t;
        // 이전 핸들러는 필요없으므로 null_mut() 사용(c로 하면 NULL과 같은 것)
        check_int_return(libc::sigaction(libc::SIGWINCH, &sigwinch_action, null_mut()))?;

        // Get the original terminal modes so we can disable raw mode on exit.
        // 현재 터미널의 속성을 termios에 읽어온다.
        let mut termios = MaybeUninit::<libc::termios>::uninit();
        check_int_return(libc::tcgetattr(STATE.stdout, termios.as_mut_ptr()))?;
        let mut termios = termios.assume_init();
        // 프로그램 종료시 터미널을 원래 상태로 복구하기 위해 현재 터미널의 속성을 저장해 놓는다.
        // termios는 복사된다.
        STATE.stdout_initial_termios = Some(termios);

        // 입력 플래그
        termios.c_iflag &= !(
            // When neither IGNBRK...
            libc::IGNBRK
            // ...nor BRKINT are set, a BREAK reads as a null byte ('\0'), ...
            | libc::BRKINT
            // ...except when PARMRK is set, in which case it reads as the sequence \377 \0 \0.
            | libc::PARMRK
            // Disable input parity checking.
            | libc::INPCK
            // Disable stripping of eighth bit.
            | libc::ISTRIP
            // Disable mapping of NL to CR on input.
            | libc::INLCR
            // Disable ignoring CR on input.
            | libc::IGNCR
            // Disable mapping of CR to NL on input.
            | libc::ICRNL
            // Disable software flow control.
            | libc::IXON
        );
        // 출력 플래그
        // Disable output processing.
        // OPOST(출력 후처리)를 꺼서 \n이 \r\n으로 자동 변환되는등의 동작을 막는다.
        termios.c_oflag &= !libc::OPOST;
        // 제어 플래그
        termios.c_cflag &= !(
            // Reset character size mask.
            libc::CSIZE
            // Disable parity generation.
            | libc::PARENB
        );
        // Set character size back to 8 bits.
        // 문자 크기를 8비트로 설정
        termios.c_cflag |= libc::CS8;
        // 로컬 플래그
        termios.c_lflag &= !(
            // Disable signal generation (SIGINT, SIGTSTP, SIGQUIT).
            // Ctrl+C 같은 키가 SIGINT 시그널을 보내는 것을 막는다.
            libc::ISIG
            // Disable canonical mode (line buffering).
            // "Canonical mode"를 비활성화.
            // 이로 인해 터미널은 Enter 키를 기다리지 않고 키가 눌리는 즉시 입력을 프로그램에 전달
            | libc::ICANON
            // Disable echoing of input characters.
            // 입력된 문자가 자동으로 화면에 표시되는 것을 막는다(에디터가 화면에 내용을 직접 그려야 함.).
            | libc::ECHO
            // Disable echoing of NL.
            | libc::ECHONL
            // Disable extended input processing (e.g. Ctrl-V).
            | libc::IEXTEN
        );

        // Set the terminal to raw mode.
        // 변경된 termios 구조체의 속성을 적용한다.
        termios.c_lflag &= !(libc::ICANON | libc::ECHO);
        check_int_return(libc::tcsetattr(STATE.stdout, libc::TCSANOW, &termios))?;

        Ok(())
    }
}

/// switch_modes() 보다 먼저 호출되기 때문에 항상 None이 리턴되는 것으로 보임
pub fn open_stdin_if_redirected() -> Option<File> {
    unsafe {
        // Did we reopen stdin during `init()`?
        if STATE.stdin != libc::STDIN_FILENO {
            Some(File::from_raw_fd(libc::STDIN_FILENO))
        } else {
            None
        }
    }
}

fn get_window_size() -> (u16, u16) {
    // 터미널의 행(row)과 열(column) 정보
    let mut winsz: libc::winsize = unsafe { mem::zeroed() };

    for attempt in 1.. {
        // libc::TIOCGWINSZ: 터미널의 창 크기를 가져오기 위한 ioctl 요청 코드
        let ret = unsafe { libc::ioctl(STATE.stdout, libc::TIOCGWINSZ, &raw mut winsz) };
        if ret == -1 || (winsz.ws_col != 0 && winsz.ws_row != 0) {
            break;
        }

        // 최대 10번만 시도
        if attempt == 10 {
            winsz.ws_col = 80;
            winsz.ws_row = 24;
            break;
        }

        // Some terminals are bad emulators and don't report TIOCGWINSZ immediately.
        thread::sleep(time::Duration::from_millis(10 * attempt));
    }

    (winsz.ws_col, winsz.ws_row)
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

/// Sets/Resets `O_NONBLOCK` on the TTY handle.
///
/// Note that setting this flag applies to both stdin and stdout, because the
/// TTY is a bidirectional device and both handles refer to the same thing.
/// 
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

fn check_int_return(ret: libc::c_int) -> apperr::Result<libc::c_int> {
    if ret < 0 {
        Err(get_last_error())
    } else {
        Ok(ret)
    }
}

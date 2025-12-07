#![feature(allocator_api)]

mod documents;
mod localization;
mod state;

use crate::state::{FormatApperr, State, StateFilePicker};
use std::{
    env,
    path::{Path, PathBuf},
    process,
};
use stdext::arena::{self, Arena, scratch_arena};
use whitedew_core::{apperr, helpers::MEBI, path, sys};

const SCRATCH_ARENA_CAPACITY: usize = 512 * MEBI;

fn main() -> process::ExitCode {
    let hook = std::panic::take_hook();
    // 오직 패닉이 발생했을 때만 호출되는 콜백을 등록
    std::panic::set_hook(Box::new(move |info| {
        drop(RestoreModes);
        drop(sys::Deinit);
        hook(info);
    }));

    match run() {
        Ok(()) => process::ExitCode::SUCCESS,
        Err(err) => {
            sys::write_stdout(&format!("{}\n", FormatApperr::from(err)));
            process::ExitCode::FAILURE
        }
    }
}

fn run() -> apperr::Result<()> {
    let _sys_deinit = sys::init();
    arena::init(SCRATCH_ARENA_CAPACITY)?;
    localization::init();

    let mut state = State::new()?;
    if handle_args(&mut state)? {
        return Ok(());
    }

    sys::switch_modes()?;
    
    Ok(())
}

fn handle_args(state: &mut State) -> apperr::Result<bool> {
    let scratch = scratch_arena(None);
    // 읽을 파일 리스트 저장소
    let mut paths: Vec<PathBuf, &Arena> = Vec::new_in(&*scratch);
    let cwd = env::current_dir()?;
    let mut dir = None;
    let mut parse_args = true;

    for arg in env::args_os().skip(1) {
        if parse_args {
            if arg == "--" {
                parse_args = false;
                continue;
            }
            if arg == "-" {
                paths.clear();
                break;
            }
            if arg == "-h" || arg == "--help" {
                print_help();
                return Ok(true);
            }
            if arg == "-v" || arg == "--version" {
                print_version();
                return Ok(true);
            }
        }

        let p = cwd.join(Path::new(&arg));
        let p = path::normalize(&p);
        if p.is_dir() {
            state.wants_file_picker = StateFilePicker::Open;
            dir = Some(p);
        } else {
            paths.push(p);
        }
    }

    for p in &paths {
        state.documents.add_file_path(p)?;
    }

    Ok(false)
}

fn print_help() {
    sys::write_stdout(concat!(
        "Usage: wd [OPTIONS] [FILE[:LINE[:COLUMN]]]\n",
        "Options:\n",
        "    -h, --help       Print this help message\n",
        "    -v, --version    Print the version number\n",
        "\n",
        "Arguments:\n",
        "    FILE[:LINE[:COLUMN]]    The file to open, optionally with line and column (e.g., foo.txt:123:45)\n",
    ));
}

fn print_version() {
    sys::write_stdout(concat!("edit version ", env!("CARGO_PKG_VERSION"), "\n"));
}

struct RestoreModes;

impl Drop for RestoreModes {
    fn drop(&mut self) {
        // \x1b[0 q: 커서 모양을 기본(default blinking block)으로 설정
        // \x1b[?25h: 커서 보이기
        // \x1b]0;\x07: 터미널 창 제목(title)을 빈 문자열로 설정
        // \x1b[?1002;1006;2004l:
        //   ?1002 → Mouse Tracking (drag events) 끄기
        //   ?1006 → SGR extended mouse mode 끄기
        //   ?2004 → Bracketed Paste Mode 끄기
        // \x1b[?1049l: 대체 스크린 버퍼(Alternate Screen Buffer) 종료
        sys::write_stdout("\x1b[0 q\x1b[?25h\x1b]0;\x07\x1b[?1002;1006;2004l\x1b[?1049l");
    }
}

#![feature(allocator_api)]

mod documents;
mod localization;
mod state;

use crate::state::{DisplayablePathBuf, FormatApperr, State, StateFilePicker};
use std::{
    env,
    path::{Path, PathBuf},
    process,
    time::Duration,
};
use stdext::arena::{self, Arena, scratch_arena};
use whitedew_core::{
    apperr,
    framebuffer::{self, IndexedColor},
    helpers::{CoordType, MEBI},
    input,
    oklab::StraightRgba,
    path,
    sys,
    unicode,
    tui::Tui,
    vt::{self, Token},
};

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

    // This will reopen stdin if it's redirected (which may fail) and switch
    // the terminal to raw mode which prevents the user from pressing Ctrl+C.
    // `handle_args` may want to print a help message (must not fail),
    // and reads files (may hang; should be cancelable with Ctrl+C).
    // As such, we call this after `handle_args`.
    sys::switch_modes()?;

    let mut vt_parser = vt::Parser::new();
    let mut input_parser = input::Parser::new();
    let mut tui = Tui::new()?;

    let _restore = setup_terminal(&mut tui, &mut state, &mut vt_parser);
    
    Ok(())
}

// Returns true if the application should exit early.
fn handle_args(state: &mut State) -> apperr::Result<bool> {
    let scratch = scratch_arena(None);
    // 읽을 파일 리스트 저장소
    let mut paths: Vec<PathBuf, &Arena> = Vec::new_in(&*scratch);
    let cwd = env::current_dir()?;
    let mut dir = None;
    let mut parse_args = true;

    // The best CLI argument parser in the world.
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

    if let Some(mut file) = sys::open_stdin_if_redirected() {
        let doc = state.documents.add_untitled()?;
        let mut tb = doc.buffer.borrow_mut();
        tb.read_file(&mut file, None)?;
        tb.mark_as_dirty();
    } else if paths.is_empty() {
        // No files were passed, and stdin is not redirected.
        state.documents.add_untitled()?;
    }

    if dir.is_none() && let Some(parent) = paths.last().and_then(|p| p.parent()) {
        // dir이 없고, 읽을 파일이 있는 경우는 읽을 파일의 parent dir을 dir로 설정
        dir = Some(parent.to_path_buf());
    }

    state.file_picker_pending_dir = DisplayablePathBuf::from_path(dir.unwrap_or(cwd));
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

/// 터미널을 TUI 앱에 적합한 모드로 전환
fn setup_terminal(tui: &mut Tui, state: &mut State, vt_parser: &mut vt::Parser) -> RestoreModes {
    // 터미널에 제어 시퀀스 전송
    sys::write_stdout(concat!(
        // 1049: Alternative Screen Buffer
        //   I put the ASB switch in the beginning, just in case the terminal performs
        //   some additional state tracking beyond the modes we enable/disable.
        // 1002: Cell Motion Mouse Tracking
        // 1006: SGR Mouse Mode
        // 2004: Bracketed Paste Mode
        // 1036: Xterm: "meta sends escape" (Alt keypresses should be encoded with ESC + char)
        //
        // 1049: 일반 화면 버퍼 대신 대체 화면 버퍼로 전환
        //       종료 시 \x1b[?1049l 하면 원래 화면으로 돌아옴
        // 1002: 마우스 이벤트 발생 시 셀 단위로 움직일 때 마다 이벤트를 보고
        // 1006: 마우스 이벤트를 SGR 포맷(\x1b[<b;x;yM / m)으로 전송하도록 한다.
        // 2004: 붙여넣기 동작을 다음과 같이 감싸서 보내줌: \x1b[200~ PASTED_TEXT \x1b[201~
        //       프로그램이 "사용자가 타이핑한 것"과 "붙여넣기"를 구분할 수 있음
        // 1036: Alt + key 입력이 ESC + key 형태로 들어오도록 강제함. 예: Alt+a → \x1b a
        "\x1b[?1049h\x1b[?1002;1006;2004h\x1b[?1036h",
        // OSC 4 color table requests for indices 0 through 15 (base colors).
        //
        // 색상 인덱스 n의 RGB 값을 요청
        // ;? 로 물음표를 넣으면 "현재 색을 알려줘"라는 의미
        // 0–7: 기본 색(블랙, 레드, 그린, …)
        // 8–15: 밝은 색 계열
        // 터미널 응답: ESC ] 4 ; index ; rgb:RRRR/GGGG/BBBB BEL
        "\x1b]4;0;?;1;?;2;?;3;?;4;?;5;?;6;?;7;?\x07",
        "\x1b]4;8;?;9;?;10;?;11;?;12;?;13;?;14;?;15;?\x07",
        // OSC 10 and 11 queries for the current foreground and background colors.
        // 전경/배경색 쿼리
        // 예 응답:
        //   ESC ] 10;rgb:aaaa/bbbb/cccc BEL
        //   ESC ] 11;rgb:0000/0000/0000 BEL
        "\x1b]10;?\x07\x1b]11;?\x07",
        // Test whether ambiguous width characters are two columns wide.
        // We use "…", because it's the most common ambiguous width character we use,
        // and the old Windows conhost doesn't actually use wcwidth, it measures the
        // actual display width of the character and assigns it columns accordingly.
        // We detect it by writing the character and asking for the cursor position.
        //
        // 모호한 너비(ambiguous-width) 문자가 1칸을 차지하는지 2칸을 차지하는지 확인
        // …(ellipsis)는 동아시아 모드에서 너비가 1 or 2 인 “ambiguous width” 문자
        // 터미널의 실제 표시 너비를 체크하기 위해:
        //   1. 커서를 행 처음으로 이동 (\r)
        //   2. … 출력
        //   3. 응답이 ;2R이면 너비=1, 응답이 ;3R이면 너비=2
        "\r…\x1b[6n",
        // CSI c reports the terminal capabilities.
        // It also helps us to detect the end of the responses, because not all
        // terminals support the OSC queries, but all of them support CSI c.
        //
        // 터미널의 capability, 버전 등을 보고
        // 예: ESC [ ? 1 ; 2 c
        // 응답이 마지막에 오기 때문에, 위의 다양한 쿼리들이 모두 끝났다는 신호로도 사용됨
        "\x1b[c",
    ));

    let mut done = false;
    let mut osc_buffer = String::new();
    let mut indexed_colors = framebuffer::DEFAULT_THEME;
    let mut color_responses = 0;
    let mut ambiguous_width = 1;

    while !done {
        let scratch = scratch_arena(None);

        // We explicitly set a high read timeout, because we're not
        // waiting for user keyboard input. If we encounter a lone ESC,
        // it's unlikely to be from a ESC keypress, but rather from a VT sequence.
        let Some(input) = sys::read_stdin(&scratch, Duration::from_secs(3)) else {
            break;
        };

        let mut vt_stream = vt_parser.parse(&input);
        while let Some(token) = vt_stream.next() {
            match token {
                Token::Csi(csi) => match csi.final_byte {
                    'c' => done = true,
                    // CPR (Cursor Position Report) response.
                    'R' => ambiguous_width = csi.params[1] as CoordType - 1,
                    _ => {}
                },
                Token::Osc { mut data, partial } => {
                    if partial {
                        osc_buffer.push_str(data);
                        continue;
                    }
                    if !osc_buffer.is_empty() {
                        osc_buffer.push_str(data);
                        data = &osc_buffer;
                    }

                    let mut splits = data.split_terminator(';');

                    let color = match splits.next().unwrap_or("") {
                        // The response is `4;<color>;rgb:<r>/<g>/<b>`.
                        "4" => match splits.next().unwrap_or("").parse::<usize>() {
                            Ok(val) if val < 16 => &mut indexed_colors[val],
                            _ => continue,
                        },
                        // The response is `10;rgb:<r>/<g>/<b>`.
                        "10" => &mut indexed_colors[IndexedColor::Foreground as usize],
                        // The response is `11;rgb:<r>/<g>/<b>`.
                        "11" => &mut indexed_colors[IndexedColor::Background as usize],
                        _ => continue,
                    };

                    let color_param = splits.next().unwrap_or("");
                    if !color_param.starts_with("rgb:") {
                        continue;
                    }

                    let mut iter = color_param[4..].split_terminator('/');
                    let rgb_parts = [(); 3].map(|_| iter.next().unwrap_or("0"));
                    let mut rgb = 0;

                    for part in rgb_parts {
                        if part.len() == 2 || part.len() == 4 {
                            let Ok(mut val) = usize::from_str_radix(part, 16) else {
                                continue;
                            };
                            if part.len() == 4 {
                                // Round from 16 bits to 8 bits.
                                val = (val * 0xff + 0x7fff) / 0xffff;
                            }
                            rgb = (rgb >> 8) | ((val as u32) << 16);
                        }
                    }

                    *color = StraightRgba::from_le(rgb | 0xff000000);
                    color_responses += 1;
                    osc_buffer.clear();
                }
                _ => {}
            }
        }
    }

    if ambiguous_width == 2 {
        // 감지된 문자 너비에 따라 유니코드 처리 방식을 설정
        unicode::setup_ambiguous_width(2);
        state.documents.reflow_all();
    }

    if color_responses == indexed_colors.len() {
        // 터미널로부터 실제 색상 정보를 얻었다면, TUI 시스템에 해당 색상표를 설정
        tui.setup_indexed_colors(indexed_colors);
    }

    RestoreModes
}

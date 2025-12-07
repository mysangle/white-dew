
use stdext::arena::scratch_arena;
use whitedew_core::{helpers::AsciiStringHelpers, sys};

// build.rs(또는 build = "...")가 실행될 때, Cargo는 임시 디렉토리를 만들어 빌드 산출물을 넣는다. OUT_DIR은 그 디렉토리 경로이다.
// include! 매크로는 그 파일의 내용을 그대로 Rust 소스 코드로 삽입
include!(concat!(env!("OUT_DIR"), "/i18n_edit.rs"));

static mut S_LANG: LangId = LangId::en;

pub fn init() {
    let scratch = scratch_arena(None);
    let langs = sys::preferred_languages(&scratch);
    let mut lang = LangId::en;

    'outer: for l in langs {
        for (prefix, id) in LANGUAGES {
            if l.starts_with_ignore_ascii_case(prefix) {
                lang = *id;
                break 'outer;
            }
        }
    }

    unsafe {
        S_LANG = lang;
    }
}

pub fn loc(id: LocId) -> &'static str {
     TRANSLATIONS[unsafe { S_LANG as usize }][id as usize]
}

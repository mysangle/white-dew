
// build.rs(또는 build = "...")가 실행될 때, Cargo는 임시 디렉토리를 만들어 빌드 산출물을 넣는다. OUT_DIR은 그 디렉토리 경로이다.
// include! 매크로는 그 파일의 내용을 그대로 Rust 소스 코드로 삽입
// build script와 같은 프로젝트에 있어야 동작
include!(concat!(env!("OUT_DIR"), "/i18n_edit.rs"));

// static mut S_LANG: LangId = LangId::en;

pub fn init() {
    
}

// pub fn loc(id: LocId) -> &'static str {
//     TRANSLATIONS[unsafe { S_LANG as usize }][id as usize]
// }

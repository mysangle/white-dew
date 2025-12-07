
use crate::sys;
use std::{alloc::AllocError, io, result};

pub const APP_ICU_MISSING: Error = Error::new_app(0);

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    // 애플리케이션에서 발생하는 오류
    App(u32),
    // ICU 라이브러리에서 발생하는 오류
    Icu(u32),
    // 시스템 수준에서 발생하는 오류
    Sys(u32),
}

impl Error {
    pub const fn new_app(code: u32) -> Self {
        Self::App(code)
    }

    pub const fn new_icu(code: u32) -> Self {
        Self::Icu(code)
    }

    pub const fn new_sys(code: u32) -> Self {
        Self::Sys(code)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        sys::io_error_to_apperr(err)
    }
}

impl From<AllocError> for Error {
    fn from(_: AllocError) -> Self {
        // TODO: Technically this breaks if the AllocError isn't recent. By then, the errno may
        // have been tained. But the stdlib AllocError is a bad type with no way to carry info.
        // 시스템의 마지막 오류 상태를 기반으로 Sys 오류 생성
        sys::get_last_error()
    }
}

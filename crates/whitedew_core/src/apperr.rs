
use std::result;

pub const APP_ICU_MISSING: Error = Error::new_app(0);

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    App(u32),
    Icu(u32),
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


use crate::documents::DocumentManager;
use whitedew_core::{apperr, icu, sys};

#[repr(transparent)]
pub struct FormatApperr(apperr::Error);

impl From<apperr::Error> for FormatApperr {
    fn from(err: apperr::Error) -> Self {
        Self(err)
    }
}

impl std::fmt::Display for FormatApperr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            //apperr::APP_ICU_MISSING => f.write_str(loc(LocId::ErrorIcuMissing)),
            apperr::Error::App(code) => write!(f, "Unknown app error code: {code}"),
            apperr::Error::Icu(code) => icu::apperr_format(f, code),
            apperr::Error::Sys(code) => sys::apperr_format(f, code),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StateFilePicker {
    None,
    Open,
    SaveAs,
    SaveAsShown, // Transitioned from SaveAs
}

pub struct State {
    pub documents: DocumentManager,
    pub wants_file_picker: StateFilePicker,
}

impl State {
    pub fn new() -> apperr::Result<Self> {
        Ok(Self {
            documents: Default::default(),
            wants_file_picker: StateFilePicker::None,
        })
    }
}

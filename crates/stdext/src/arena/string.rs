
use std::ops::Deref;
use super::Arena;

#[derive(Clone)]
pub struct ArenaString<'a> {
    vec: Vec<u8, &'a Arena>,
}

impl Deref for ArenaString<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> ArenaString<'a> {
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.vec.as_slice()) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8, &'a Arena>) -> Self {
        Self { vec: bytes }
    }
}

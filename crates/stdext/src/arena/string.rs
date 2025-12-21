
use crate::helpers::ReplaceRange;
use std::{
    fmt,
    ops::{Bound, Deref, RangeBounds},
};
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
    /// Creates a new [`ArenaString`] in the given arena.
    #[must_use]
    pub const fn new_in(arena: &'a Arena) -> Self {
        Self { vec: Vec::new_in(arena) }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.vec.as_slice()) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn from_utf8_unchecked(bytes: Vec<u8, &'a Arena>) -> Self {
        Self { vec: bytes }
    }

    /// Turns a [`Vec<u8>`] into an [`ArenaString`], replacing invalid UTF-8 sequences with U+FFFD.
    #[must_use]
    pub fn from_utf8_lossy_owned(v: Vec<u8, &'a Arena>) -> Self {
        match Self::from_utf8_lossy(v.allocator(), &v) {
            Ok(..) => unsafe { Self::from_utf8_unchecked(v) },
            Err(s) => s,
        }
    }

    /// Checks whether `text` contains only valid UTF-8.
    /// If the entire string is valid, it returns `Ok(text)`.
    /// Otherwise, it returns `Err(ArenaString)` with all invalid sequences replaced with U+FFFD.
    pub fn from_utf8_lossy<'s>(arena: &'a Arena, text: &'s [u8]) -> Result<&'s str, Self> {
        // 유효하지 않은 부분(다음 유효한 청크 이전)까지의 청크에 대한 이터레이터
        let mut iter = text.utf8_chunks();
        let Some(mut chunk) = iter.next() else {
            return Ok("");
        };

        let valid = chunk.valid();
        if chunk.invalid().is_empty() {
            // 전체 입력이 유효한 UTF-8
            debug_assert_eq!(valid.len(), text.len());
            return Ok(unsafe { str::from_utf8_unchecked(text) });
        }

        // 유효하지 않은 문자를 포함하고 있는 경우
        const REPLACEMENT: &str = "\u{FFFD}";

        let mut res = Self::new_in(arena);
        res.reserve(text.len());

        loop {
            res.push_str(chunk.valid());
            if !chunk.invalid().is_empty() {
                 // 유효하지 않은 부분은 통째로 REPLACEMENT로 변경한다.
                res.push_str(REPLACEMENT);
            }
            chunk = match iter.next() {
                Some(chunk) => chunk,
                None => break,
            };
        }

        // 유효하지 않은 부분은 REPLACEMENT로 변경된 text를 리턴한다.
        Err(res)
    }

    /// Reserves *additional* memory. For you old folks out there (totally not me),
    /// this is different from C++'s `reserve` which reserves a total size.
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional)
    }

    /// Append some text.
    pub fn push_str(&mut self, string: &str) {
        self.vec.extend_from_slice(string.as_bytes())
    }

    /// Returns a mutable reference to the contents of this `String`.
    ///
    /// # Safety
    ///
    /// The underlying `&mut Vec` allows writing bytes which are not valid UTF-8.
    pub unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8, &'a Arena> {
        &mut self.vec
    }

    /// Replaces a range of characters with a new string.
    pub fn replace_range<R: RangeBounds<usize>>(&mut self, range: R, replace_with: &str) {
        // 유효한 UTF-8 바이트 시퀀스임을 보장하기 위한 확인
        match range.start_bound() {
            Bound::Included(&n) => assert!(self.is_char_boundary(n)),
            Bound::Excluded(&n) => assert!(self.is_char_boundary(n + 1)),
            Bound::Unbounded => {}
        };
        match range.end_bound() {
            Bound::Included(&n) => assert!(self.is_char_boundary(n + 1)),
            Bound::Excluded(&n) => assert!(self.is_char_boundary(n)),
            Bound::Unbounded => {}
        };
        unsafe { self.as_mut_vec() }.replace_range(range, replace_with.as_bytes());
    }

    /// Now it's small! Alarming!
    ///
    /// *Do not* call this unless this string is the last thing on the arena.
    /// Arenas are stacks, they can't deallocate what's in the middle.
    pub fn shrink_to_fit(&mut self) {
        self.vec.shrink_to_fit()
    }
}

impl fmt::Write for ArenaString<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! arena_format {
    ($arena:expr, $($arg:tt)*) => {{
        use std::fmt::Write as _;
        let mut output = stdext::arena::ArenaString::new_in($arena);
        output.write_fmt(format_args!($($arg)*)).unwrap();
        output
    }}
}


pub const KILO: usize = 1000;
pub const MEGA: usize = 1000 * 1000;
pub const GIGA: usize = 1000 * 1000 * 1000;

pub const KIBI: usize = 1024;
pub const MEBI: usize = 1024 * 1024;
pub const GIBI: usize = 1024 * 1024 * 1024;

pub trait AsciiStringHelpers {
    /// Tests if a string starts with a given ASCII prefix.
    ///
    /// This function name really is a mouthful, but it's a combination
    /// of [`str::starts_with`] and [`str::eq_ignore_ascii_case`].
    fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool;
}

impl AsciiStringHelpers for str {
    fn starts_with_ignore_ascii_case(&self, prefix: &str) -> bool {
        // Casting to bytes first ensures we skip any UTF8 boundary checks.
        // Since the comparison is ASCII, we don't need to worry about that.
        let s = self.as_bytes();
        let p = prefix.as_bytes();
        p.len() <= s.len() && s[..p.len()].eq_ignore_ascii_case(p)
    }
}

/// A viewport coordinate type used throughout the application.
pub type CoordType = isize;

/// To avoid overflow issues because you're adding two [`CoordType::MAX`]
/// values together, you can use [`COORD_TYPE_SAFE_MAX`] instead.
///
/// It equates to half the bits contained in [`CoordType`], which
/// for instance is 32767 (0x7FFF) when [`CoordType`] is a [`i32`].
pub const COORD_TYPE_SAFE_MAX: CoordType = (1 << (CoordType::BITS / 2 - 1)) - 1;

/// A 2D point. Uses [`CoordType`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: CoordType,
    pub y: CoordType,
}

impl Point {
    pub const MIN: Self = Self { x: CoordType::MIN, y: CoordType::MIN };
    pub const MAX: Self = Self { x: CoordType::MAX, y: CoordType::MAX };
}

/// A 2D rectangle. Uses [`CoordType`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: CoordType,
    pub top: CoordType,
    pub right: CoordType,
    pub bottom: CoordType,
}

// A 2D size. Uses [`CoordType`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: CoordType,
    pub height: CoordType,
}

impl Size {
    pub fn as_rect(&self) -> Rect {
        Rect { left: 0, top: 0, right: self.width, bottom: self.height }
    }
}

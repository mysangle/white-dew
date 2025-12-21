
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

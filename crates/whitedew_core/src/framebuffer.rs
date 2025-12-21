
use crate::oklab::StraightRgba;

/// Number of indices used by [`IndexedColor`].
pub const INDEXED_COLORS_COUNT: usize = 18;

/// Fallback theme. Matches Windows Terminal's Ottosson theme.
pub const DEFAULT_THEME: [StraightRgba; INDEXED_COLORS_COUNT] = [
    StraightRgba::from_be(0x000000ff), // Black
    StraightRgba::from_be(0xbe2c21ff), // Red
    StraightRgba::from_be(0x3fae3aff), // Green
    StraightRgba::from_be(0xbe9a4aff), // Yellow
    StraightRgba::from_be(0x204dbeff), // Blue
    StraightRgba::from_be(0xbb54beff), // Magenta
    StraightRgba::from_be(0x00a7b2ff), // Cyan
    StraightRgba::from_be(0xbebebeff), // White
    StraightRgba::from_be(0x808080ff), // BrightBlack
    StraightRgba::from_be(0xff3e30ff), // BrightRed
    StraightRgba::from_be(0x58ea51ff), // BrightGreen
    StraightRgba::from_be(0xffc944ff), // BrightYellow
    StraightRgba::from_be(0x2f6affff), // BrightBlue
    StraightRgba::from_be(0xfc74ffff), // BrightMagenta
    StraightRgba::from_be(0x00e1f0ff), // BrightCyan
    StraightRgba::from_be(0xffffffff), // BrightWhite
    // --------
    StraightRgba::from_be(0x000000ff), // Background
    StraightRgba::from_be(0xbebebeff), // Foreground
];

/// Standard 16 VT & default foreground/background colors.
#[derive(Clone, Copy)]
pub enum IndexedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    Background,
    Foreground,
}

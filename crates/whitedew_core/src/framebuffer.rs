
use crate::oklab::StraightRgba;
use std::cell::Cell;

const HASH_MULTIPLIER: usize = 6364136223846793005; // Knuth's MMIX multiplier
/// The size of our cache table. 1<<8 = 256.
const CACHE_TABLE_LOG2_SIZE: usize = 8;
const CACHE_TABLE_SIZE: usize = 1 << CACHE_TABLE_LOG2_SIZE;
/// To index into the cache table, we use `color * HASH_MULTIPLIER` as the hash.
/// Since the multiplication "shifts" the bits up, we don't just mask the lowest
/// 8 bits out, but rather shift 56 bits down to get the best bits from the top.
const CACHE_TABLE_SHIFT: usize = usize::BITS as usize - CACHE_TABLE_LOG2_SIZE;

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

/// A shoddy framebuffer for terminal applications.
///
/// The idea is that you create a [`Framebuffer`], draw a bunch of text and
/// colors into it, and it takes care of figuring out what changed since the
/// last rendering and sending the differences as VT to the terminal.
///
/// This is an improvement over how many other terminal applications work,
/// as they fail to accurately track what changed. If you watch the output
/// of `vim` for instance, you'll notice that it redraws unrelated parts of
/// the screen all the time.
pub struct Framebuffer {
    /// Store the color palette.
    indexed_colors: [StraightRgba; INDEXED_COLORS_COUNT],
    background_fill: StraightRgba,
    foreground_fill: StraightRgba,
    /// The colors used for `contrast()`. It stores the default colors
    /// of the palette as [dark, light], unless the palette is recognized
    /// as a light them, in which case it swaps them.
    auto_colors: [StraightRgba; 2],
    /// A cache table for previously contrasted colors.
    /// See: <https://fgiesen.wordpress.com/2019/02/11/cache-tables/>
    contrast_colors: [Cell<(StraightRgba, StraightRgba)>; CACHE_TABLE_SIZE],
}

impl Framebuffer {
    /// Creates a new framebuffer.
    pub fn new() -> Self {
        Self {
            indexed_colors: DEFAULT_THEME,
            background_fill: DEFAULT_THEME[IndexedColor::Background as usize],
            foreground_fill: DEFAULT_THEME[IndexedColor::Foreground as usize],
            auto_colors: [
                DEFAULT_THEME[IndexedColor::Black as usize],
                DEFAULT_THEME[IndexedColor::BrightWhite as usize],
            ],
            contrast_colors: [const { Cell::new((StraightRgba::zero(), StraightRgba::zero())) }; CACHE_TABLE_SIZE],
        }
    }

    /// Sets the base color palette.
    ///
    /// If you call this method, [`Framebuffer`] expects that you
    /// successfully detect the light/dark mode of the terminal.
    pub fn set_indexed_colors(&mut self, colors: [StraightRgba; INDEXED_COLORS_COUNT]) {
        self.indexed_colors = colors;
        self.background_fill = StraightRgba::zero();
        self.foreground_fill = StraightRgba::zero();

        self.auto_colors = [
            self.indexed_colors[IndexedColor::Black as usize],
            self.indexed_colors[IndexedColor::BrightWhite as usize],
        ];
        if !Self::is_dark(self.auto_colors[0]) {
            self.auto_colors.swap(0, 1);
        }
    }

    fn is_dark(color: StraightRgba) -> bool {
        color.as_oklab().lightness() < 0.5
    }

    #[inline]
    pub fn indexed(&self, index: IndexedColor) -> StraightRgba {
        self.indexed_colors[index as usize]
    }

    /// Returns a color from the palette.
    ///
    /// To facilitate constant folding by the compiler,
    /// alpha is given as a fraction (`numerator` / `denominator`).
    #[inline]
    pub fn indexed_alpha(
        &self,
        index: IndexedColor,
        numerator: u32,
        denominator: u32,
    ) -> StraightRgba {
        let c = self.indexed_colors[index as usize].to_le();
        let a = 255 * numerator / denominator;
        StraightRgba::from_le(a << 24 | (c & 0x00ffffff))
    }

    /// Returns a color opposite to the brightness of the given `color`.
    pub fn contrasted(&self, color: StraightRgba) -> StraightRgba {
        let idx = (color.to_ne() as usize).wrapping_mul(HASH_MULTIPLIER) >> CACHE_TABLE_SHIFT;
        let slot = self.contrast_colors[idx].get();
        if slot.0 == color {
            // 캐시된 칼라 사용
            slot.1
        } else {
            self.contrasted_slow(color)
        }
    }

    /// contrast 칼라 계산 후 캐시
    /// 어두운 배경에는 밝은 글씨(BrightWhite)를, 밝은 배경에는 어두운 글씨(Black)
    #[cold]
    fn contrasted_slow(&self, color: StraightRgba) -> StraightRgba {
        let idx = (color.to_ne() as usize).wrapping_mul(HASH_MULTIPLIER) >> CACHE_TABLE_SHIFT;
        let contrast = self.auto_colors[Self::is_dark(color) as usize];
        self.contrast_colors[idx].set((color, contrast));
        contrast
    }
}

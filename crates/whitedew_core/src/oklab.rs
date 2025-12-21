
/// A sRGB color with straight (= not premultiplied) alpha.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct StraightRgba(u32);

impl StraightRgba {
    #[inline]
    pub const fn from_be(color: u32) -> Self {
        StraightRgba(u32::from_be(color))
    }
}

impl StraightRgba {
    #[inline]
    pub const fn from_le(color: u32) -> Self {
        StraightRgba(u32::from_le(color))
    }
}

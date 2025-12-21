
use crate::{
    apperr,
    framebuffer::INDEXED_COLORS_COUNT,
    oklab::StraightRgba,
};

pub struct Tui {

}

impl Tui {
    pub fn new() -> apperr::Result<Self> {
        let mut tui = Self {

        };

        Ok(tui)
    }

    /// Sets up the framebuffer's color palette.
    pub fn setup_indexed_colors(&mut self, colors: [StraightRgba; INDEXED_COLORS_COUNT]) {

    }
}

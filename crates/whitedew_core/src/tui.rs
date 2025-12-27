
use crate::{
    apperr,
    clipboard::Clipboard,
    framebuffer::{Framebuffer, IndexedColor, INDEXED_COLORS_COUNT},
    helpers::Size,
    input,
    oklab::StraightRgba,
};
use std::{mem, time};
use stdext::arena::{Arena, ArenaString};

type Input<'input> = input::Input<'input>;
type InputKey = input::InputKey;
type InputMouseState = input::InputMouseState;

/// In order for the TUI to show the correct Ctrl/Alt/Shift
/// translations, this struct lets you set them.
pub struct ModifierTranslations {
    pub ctrl: &'static str,
    pub alt: &'static str,
    pub shift: &'static str,
}

pub struct Tui {
    /// The framebuffer used for rendering.
    framebuffer: Framebuffer,
    modifier_translations: ModifierTranslations,
    floater_default_bg: StraightRgba,
    floater_default_fg: StraightRgba,
    modal_default_bg: StraightRgba,
    modal_default_fg: StraightRgba,
    read_timeout: time::Duration,
    settling_have: i32,
    settling_want: i32,
    /// Last known terminal size.
    ///
    /// This lives here instead of [`Context`], because we need to
    /// track the state across frames and input events.
    /// This also applies to the remaining members in this block below.
    size: Size,
    /// The clipboard contents.
    clipboard: Clipboard,
}

impl Tui {
    pub fn new() -> apperr::Result<Self> {
        let mut tui = Self {
            framebuffer: Framebuffer::new(),
            modifier_translations: ModifierTranslations {
                ctrl: "Ctrl",
                alt: "Alt",
                shift: "Shift",
            },
            floater_default_bg: StraightRgba::zero(),
            floater_default_fg: StraightRgba::zero(),
            modal_default_bg: StraightRgba::zero(),
            modal_default_fg: StraightRgba::zero(),
            read_timeout: time::Duration::MAX,
            settling_have: 0,
            settling_want: 0,
            size: Size { width: 0, height: 0 },
            clipboard: Default::default(),
        };

        Ok(tui)
    }

    /// Sets up the framebuffer's color palette.
    pub fn setup_indexed_colors(&mut self, colors: [StraightRgba; INDEXED_COLORS_COUNT]) {
        self.framebuffer.set_indexed_colors(colors);
    }

    /// Returns an indexed color from the framebuffer.
    #[inline]
    pub fn indexed(&self, index: IndexedColor) -> StraightRgba {
        self.framebuffer.indexed(index)
    }

    /// Returns an indexed color from the framebuffer with the given alpha.
    /// See [`Framebuffer::indexed_alpha()`].
    #[inline]
    pub fn indexed_alpha(
        &self,
        index: IndexedColor,
        numerator: u32,
        denominator: u32,
    ) -> StraightRgba {
        self.framebuffer.indexed_alpha(index, numerator, denominator)
    }

    // Returns a color in contrast with the given color.
    /// 시각적으로 대비되는 색상 반환
    /// See [`Framebuffer::contrasted()`].
    pub fn contrasted(&self, color: StraightRgba) -> StraightRgba {
        self.framebuffer.contrasted(color)
    }

    /// Set up translations for Ctrl/Alt/Shift modifiers.
    pub fn setup_modifier_translations(&mut self, translations: ModifierTranslations) {
        self.modifier_translations = translations;
    }

    /// Set the default background color for floaters (dropdowns, etc.).
    pub fn set_floater_default_bg(&mut self, color: StraightRgba) {
        self.floater_default_bg = color;
    }

    /// Set the default foreground color for floaters (dropdowns, etc.).
    pub fn set_floater_default_fg(&mut self, color: StraightRgba) {
        self.floater_default_fg = color;
    }

    /// Set the default background color for modals.
    pub fn set_modal_default_bg(&mut self, color: StraightRgba) {
        self.modal_default_bg = color;
    }

    /// Set the default foreground color for modals.
    pub fn set_modal_default_fg(&mut self, color: StraightRgba) {
        self.modal_default_fg = color;
    }

    /// If the TUI is currently running animations, etc.,
    /// this will return a timeout smaller than [`time::Duration::MAX`].
    pub fn read_timeout(&mut self) -> time::Duration {
        mem::replace(&mut self.read_timeout, time::Duration::MAX)
    }

    /// Returns the viewport size.
    pub fn size(&self) -> Size {
        // We don't use the size stored in the framebuffer, because until
        // `render()` is called, the framebuffer will use a stale size.
        self.size
    }

    /// Returns the clipboard (mutable).
    pub fn clipboard_mut(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }

    /// Starts a new frame and returns a [`Context`] for it.
    pub fn create_context<'a, 'input>(
        &'a mut self,
        input: Option<Input<'input>>,
    ) -> Context<'a, 'input> {

    }

    /// After you finished processing all input, continue redrawing your UI until this returns false.
    pub fn needs_settling(&mut self) -> bool {
        self.settling_have <= self.settling_want
    }

    /// Renders the last frame into the framebuffer and returns the VT output.
    pub fn render<'a>(&mut self, arena: &'a Arena) -> ArenaString<'a> {

    }
}

/// Context is a temporary object that is created for each frame.
/// Its primary purpose is to build a UI tree.
pub struct Context<'a, 'input> {
    tui: &'a mut Tui,
    /// Current text input, if any.
    input_text: Option<&'input str>,
}

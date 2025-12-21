
use crate::{apperr, cell::SemiRefCell};
use std::{fs::File, rc::Rc};

pub type TextBufferCell = SemiRefCell<TextBuffer>;

pub type RcTextBuffer = Rc<TextBufferCell>;

pub struct TextBuffer {

}

impl TextBuffer {
    pub fn new_rc(small: bool) -> apperr::Result<RcTextBuffer> {
        let buffer = Self::new(small)?;
        Ok(Rc::new(SemiRefCell::new(buffer)))
    }

    pub fn new(small: bool) -> apperr::Result<Self> {
        Ok(Self {
            
        })
    }

    pub fn read_file(&mut self, file: &mut File, encoding: Option<&'static str>) -> apperr::Result<()> {
        Ok(())
    }

    pub fn mark_as_dirty(&mut self) {

    }
}

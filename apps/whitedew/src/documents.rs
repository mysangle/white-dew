
use std::{collections::LinkedList, path::Path};
use whitedew_core::{
    apperr,
    buffer::{RcTextBuffer, TextBuffer},
};

#[derive(Default)]
pub struct DocumentManager {
    list: LinkedList<Document>,
}

impl DocumentManager {
    #[inline]
    pub fn active(&self) -> Option<&Document> {
        self.list.front()
    }
    
    pub fn add_file_path(&mut self, path: &Path) -> apperr::Result<&mut Document> {
        let buffer = Self::create_buffer()?;

        let mut doc = Document {
            buffer,
            filename: Default::default(),
        };

        self.list.push_front(doc);
        Ok(self.list.front_mut().unwrap())
    }

    pub fn add_untitled(&mut self) -> apperr::Result<&mut Document> {
        let buffer = Self::create_buffer()?;
        let mut doc = Document {
            buffer,
            filename: Default::default(),
        };

        self.list.push_front(doc);
        Ok(self.list.front_mut().unwrap())
    }

    fn create_buffer() -> apperr::Result<RcTextBuffer> {
        let buffer = TextBuffer::new_rc(false)?;

        Ok(buffer)
    }

    pub fn reflow_all(&self) {

    }
}

pub struct Document {
    pub buffer: RcTextBuffer,
    pub filename: String,
}


use std::{collections::LinkedList, path::Path};
use whitedew_core::apperr;

#[derive(Default)]
pub struct DocumentManager {
    list: LinkedList<Document>,
}

impl DocumentManager {
    pub fn add_file_path(&mut self, path: &Path) -> apperr::Result<&mut Document> {
        let mut doc = Document {};

        self.list.push_front(doc);
        Ok(self.list.front_mut().unwrap())
    }
}

pub struct Document {

}

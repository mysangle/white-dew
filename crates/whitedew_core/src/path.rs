
use std::path::{Path, PathBuf};

pub fn normalize(path: &Path) -> PathBuf {
    let mut res = PathBuf::with_capacity(path.as_os_str().as_encoded_bytes().len());
    let mut root_len = 0;

    res
}

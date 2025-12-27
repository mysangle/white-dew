
/// The builtin, internal clipboard of the editor.
///
/// This is useful particularly when the terminal doesn't support
/// OSC 52 or when the clipboard contents are huge (e.g. 1GiB).
#[derive(Default)]
pub struct Clipboard {
    data: Vec<u8>,
    line_copy: bool,
    wants_host_sync: bool,
}

impl Clipboard {
    /// Returns the current contents of the clipboard.
    pub fn read(&self) -> &[u8] {
        &self.data
    }
}

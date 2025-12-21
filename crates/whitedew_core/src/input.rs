
/// Parses VT sequences into input events.
pub struct Parser {
    bracketed_paste: bool,
    bracketed_paste_buf: Vec<u8>,
    x10_mouse_want: bool,
    x10_mouse_buf: [char; 3],
    x10_mouse_len: usize,
}

impl Parser {
    /// Creates a new parser that turns VT sequences into input events.
    /// Keep the instance alive for the lifetime of the input stream.
    pub fn new() -> Self {
        Self {
            bracketed_paste: false,
            bracketed_paste_buf: Vec::new(),
            x10_mouse_want: false,
            x10_mouse_buf: ['\0'; 3],
            x10_mouse_len: 0,
        }
    }
}

//! The iOS stand-in for the `arboard` API that `SystemClipboard` uses.
//!
//! For now no system clipboard is reachable, so `SystemClipboard` keeps the
//! text copied in the game and pastes it back in the game. UIPasteboard comes
//! later, behind these same three calls.

/// Why the system clipboard could not be used.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    /// The clipboard holds no text.
    ContentNotAvailable,
    /// The system clipboard is not reachable.
    ClipboardNotSupported,
}

/// The system clipboard.
pub struct Clipboard;

impl Clipboard {
    pub fn new() -> Result<Self, Error> {
        Err(Error::ClipboardNotSupported)
    }
    pub fn get_text(&mut self) -> Result<String, Error> {
        Err(Error::ContentNotAvailable)
    }
    pub fn set_text(&mut self, _text: String) -> Result<(), Error> {
        Err(Error::ClipboardNotSupported)
    }
}

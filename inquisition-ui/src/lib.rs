//! A widget based cli ui rendering library
use std::ops::{Deref, DerefMut};

pub use sync_input::{Input, Prompt};
pub use widget::Widget;

/// In build widgets
pub mod widgets {
    pub use super::char_input::CharInput;
    pub use super::list::{List, ListPicker};
    pub use super::string_input::StringInput;
    pub use super::text::Text;

    /// The default type for filter_map_char in [`StringInput`] and [`CharInput`]
    pub type FilterMapChar = fn(char) -> Option<char>;

    /// Character filter that lets every character through
    pub fn no_filter(c: char) -> Option<char> {
        Some(c)
    }
}

cfg_async! {
pub use async_input::AsyncPrompt;
mod async_input;
}

pub mod backend;
mod char_input;
pub mod error;
pub mod events;
mod list;
mod string_input;
mod sync_input;
mod text;
mod widget;

/// Returned by [`Prompt::validate`]
pub enum Validation {
    /// If the prompt is ready to finish.
    Finish,
    /// If the state is valid, but the prompt should still persist.
    /// Unlike returning an Err, this will not print anything unique, and is a way for the prompt to
    /// say that it internally has processed the `Enter` key, but is not complete.
    Continue,
}

/// Assume the highlighted part of the block below is the place available for rendering
/// in the given box
/// ```text
///  ____________
/// |            |
/// |     ███████|
/// |  ██████████|
/// |  ██████████|
/// '------------'
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct Layout {
    /// ```text
    ///  ____________
    /// |  vvv-- line_offset
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub line_offset: u16,
    /// ```text
    ///  ____________
    /// |vv-- offset_x
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub offset_x: u16,
    /// ```text
    ///  .-- offset_y
    /// |'>          |
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub offset_y: u16,
    /// ```text
    ///  ____________
    /// |  vvvvvvvvvv-- width
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub width: u16,
    /// ```text
    ///  ____________
    /// |.-- height  |
    /// |'>   ███████|
    /// |'>██████████|
    /// |'>██████████|
    /// '------------'
    /// ```
    pub height: u16,
}

impl Layout {
    pub fn new(line_offset: u16, terminal_size: backend::Size) -> Self {
        Self {
            line_offset,
            offset_x: 0,
            offset_y: 0,
            width: terminal_size.width,
            height: terminal_size.height,
        }
    }

    pub fn with_line_offset(mut self, line_offset: u16) -> Self {
        self.line_offset = line_offset;
        self
    }

    pub fn with_terminal_size(mut self, terminal_size: backend::Size) -> Self {
        self.set_terminal_size(terminal_size);
        self
    }

    pub fn with_offset(mut self, offset_x: u16, offset_y: u16) -> Self {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
        self
    }

    pub fn set_terminal_size(&mut self, terminal_size: backend::Size) {
        self.width = terminal_size.width;
        self.height = terminal_size.height;
    }

    pub fn line_width(&self) -> u16 {
        self.width - self.line_offset - self.offset_x
    }
}

struct TerminalState<B: backend::Backend> {
    backend: B,
    hide_cursor: bool,
    enabled: bool,
}

impl<B: backend::Backend> TerminalState<B> {
    fn new(backend: B, hide_cursor: bool) -> Self {
        Self {
            backend,
            enabled: false,
            hide_cursor,
        }
    }

    fn init(&mut self) -> error::Result<()> {
        self.enabled = true;
        if self.hide_cursor {
            self.backend.hide_cursor()?;
        }
        self.backend.enable_raw_mode()
    }

    fn reset(&mut self) -> error::Result<()> {
        self.enabled = false;
        if self.hide_cursor {
            self.backend.show_cursor()?;
        }
        self.backend.disable_raw_mode()
    }
}

impl<B: backend::Backend> Drop for TerminalState<B> {
    fn drop(&mut self) {
        if self.enabled {
            let _ = self.reset();
        }
    }
}

impl<B: backend::Backend> Deref for TerminalState<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: backend::Backend> DerefMut for TerminalState<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "tokio", feature = "async-std", feature = "smol"))]
            $item
        )*
    };
}

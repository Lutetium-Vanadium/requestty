//! A module containing the in-built widgets and types required by them

use std::io;

use textwrap::{core::Fragment, WordSeparator};

use crate::{backend::Backend, events::KeyEvent, layout::Layout};

pub use crate::char_input::CharInput;
pub use crate::prompt::{Delimiter, Prompt};
pub use crate::select::{List, Select};
pub use crate::string_input::StringInput;
pub use crate::text::Text;

/// The default type for `filter_map` in [`StringInput`] and [`CharInput`]
pub type FilterMapChar = fn(char) -> Option<char>;

/// Character filter that lets every character through
pub(crate) fn no_filter(c: char) -> Option<char> {
    Some(c)
}

/// A trait to represent renderable objects.
///
/// There are 2 purposes of a widget.
/// 1. Rendering to the screen.
/// 2. Handling input events.
///
/// # Render Cycle
///
/// Rendering happens in a 3 step process.
/// 1. First, the height is calculated with the [`height`] function.
/// 2. Then, the [`render`] function is called which is where the actual drawing happens. The
///    cursor should end at the position reflected by the layout.
/// 3. Finally, the cursor position which the user needs should see is calculated with the
///    [`cursor_pos`] function.
///
/// While it is not a guarantee that the terminal will be in raw mode, it is highly recommended that
/// those implementing the render cycle call render while in raw mode.
///
/// [`height`]: Widget::height
/// [`render`]: Widget::render
/// [`cursor_pos`]: Widget::cursor_pos
pub trait Widget {
    /// Render to a given backend.
    ///
    /// The widget is responsible for updating the layout to reflect the space that it has used.
    fn render<B: Backend>(&mut self, layout: &mut Layout, backend: &mut B) -> io::Result<()>;

    /// The number of rows of the terminal the widget will take when rendered.
    ///
    /// The widget is responsible for updating the layout to reflect the space that it will use.
    fn height(&mut self, layout: &mut Layout) -> u16;

    /// The position of the cursor to be placed at after render. The returned value should be in the
    /// form of (x, y), with (0, 0) being the top left of the screen.
    ///
    /// For example, if you want the cursor to be at the first character that could be printed,
    /// `cursor_pos` would be `(layout.offset_x + layout.line_offset, layout.offset_y)`. Also see
    /// [`Layout::offset_cursor`].
    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16);

    /// Handle a key input. It should return whether key was handled.
    fn handle_key(&mut self, key: KeyEvent) -> bool;
}

impl<T: std::ops::Deref<Target = str> + ?Sized> Widget for T {
    /// Does not allow multi-line strings. If the string requires more than a single line, it adds
    /// cuts it short and adds '...' to the end.
    ///
    /// If a multi-line string is required, use the [`Text`](crate::widgets::Text) widget.
    fn render<B: Backend>(&mut self, layout: &mut Layout, backend: &mut B) -> io::Result<()> {
        let max_width = layout.line_width() as usize;

        layout.offset_y += 1;
        layout.line_offset = 0;

        if max_width <= 3 {
            for _ in 0..max_width {
                backend.write_all(b".")?;
            }
        } else if textwrap::core::display_width(self) > max_width {
            let mut width = 0;
            let mut prev_whitespace_len = 0;
            let max_width = max_width - 3; // leave space for the '...'

            for word in WordSeparator::UnicodeBreakProperties.find_words(self) {
                width += word.width() as usize + prev_whitespace_len;
                if width > max_width {
                    break;
                }

                // Write out the whitespace only if the next word can also fit
                for _ in 0..prev_whitespace_len {
                    backend.write_all(b" ")?;
                }
                backend.write_all(word.as_bytes())?;

                prev_whitespace_len = word.whitespace_width() as usize;
            }

            backend.write_all(b"...")?;
        } else {
            backend.write_all(self.as_bytes())?;
        }

        backend
            .move_cursor_to(layout.offset_x, layout.offset_y)
            .map_err(Into::into)
    }

    /// Does not allow multi-line strings.
    ///
    /// If a multi-line string is required, use the [`Text`](crate::widgets::Text) widget.
    fn height(&mut self, layout: &mut Layout) -> u16 {
        layout.offset_y += 1;
        layout.line_offset = 0;
        1
    }

    /// Returns the location of the first character
    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        layout.offset_cursor((layout.line_offset, 0))
    }

    /// This widget does not handle any events
    fn handle_key(&mut self, _: crate::events::KeyEvent) -> bool {
        false
    }
}

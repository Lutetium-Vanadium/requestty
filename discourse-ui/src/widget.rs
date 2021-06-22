use textwrap::core::Fragment;

use crate::{backend::Backend, error, events::KeyEvent, layout::Layout};

/// A trait to represent renderable objects.
pub trait Widget {
    /// Render to a given backend. The widget is responsible for updating the layout to reflect the
    /// space that it has used.
    fn render<B: Backend>(&mut self, layout: &mut Layout, backend: &mut B) -> error::Result<()>;

    /// The number of rows of the terminal the widget will take when rendered. The widget is
    /// responsible for updating the layout to reflect the space that it will use.
    fn height(&mut self, layout: &mut Layout) -> u16;

    /// The position of the cursor to end at (x, y), with (0,0) being the start of the input
    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16);

    /// Handle a key input. It should return whether key was handled.
    fn handle_key(&mut self, key: KeyEvent) -> bool;
}

impl<T: std::ops::Deref<Target = str>> Widget for T {
    /// Does not allow multi-line strings
    fn render<B: Backend>(&mut self, layout: &mut Layout, backend: &mut B) -> error::Result<()> {
        let max_width = layout.line_width() as usize;

        if max_width <= 3 {
            return Err(std::fmt::Error.into());
        }

        layout.offset_y += 1;
        layout.line_offset = 0;

        if textwrap::core::display_width(self) > max_width {
            let mut width = 0;
            let mut prev_whitespace_len = 0;
            let max_width = max_width - 3; // leave space for the '...'

            for word in textwrap::core::find_words(self) {
                width += word.width() + prev_whitespace_len;
                if width > max_width {
                    break;
                }

                // Write out the whitespace only if the next word can also fit
                for _ in 0..prev_whitespace_len {
                    backend.write_all(b" ")?;
                }
                backend.write_all(word.as_bytes())?;

                prev_whitespace_len = word.whitespace_width();
            }

            backend.write_all(b"...")?;
            backend.move_cursor_to(layout.offset_x, layout.offset_y)
        } else {
            backend.write_all(self.as_bytes())?;
            backend.move_cursor_to(layout.offset_x, layout.offset_y)
        }
    }

    /// Does not allow multi-line strings
    fn height(&mut self, layout: &mut Layout) -> u16 {
        layout.offset_y += 1;
        1
    }

    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        (layout.line_offset, 0)
    }

    fn handle_key(&mut self, _: KeyEvent) -> bool {
        false
    }
}

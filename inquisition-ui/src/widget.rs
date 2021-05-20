use textwrap::core::Fragment;

use crate::{backend::Backend, error, events::KeyEvent, Layout};

/// A trait to represent renderable objects.
pub trait Widget {
    /// Handle a key input. It should return whether key was handled.
    #[allow(unused_variables)]
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        false
    }

    /// Render to stdout. `max_width` is the number of characters that can be printed in the current
    /// line.
    fn render<B: Backend>(
        &mut self,
        layout: Layout,
        backend: &mut B,
    ) -> error::Result<()>;

    /// The number of rows of the terminal the widget will take when rendered
    fn height(&mut self, layout: Layout) -> u16;

    /// The position of the cursor to end at, with (0,0) being the start of the input
    #[allow(unused_variables)]
    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        (layout.line_offset, 0)
    }
}

impl Widget for &str {
    /// Does not allow multi-line strings
    fn render<B: Backend>(
        &mut self,
        layout: Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        let max_width = layout.line_width() as usize;

        if max_width <= 3 {
            return Err(std::fmt::Error.into());
        }

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

            backend.write_all(b"...").map_err(Into::into)
        } else {
            backend.write_all(self.as_bytes()).map_err(Into::into)
        }
    }

    /// Does not allow multi-line strings
    fn height(&mut self, _: Layout) -> u16 {
        0
    }
}

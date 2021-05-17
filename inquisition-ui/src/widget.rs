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

impl<T: AsRef<str>> Widget for T {
    /// Does not allow multi-line strings
    fn render<B: Backend>(
        &mut self,
        layout: Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        let s = self.as_ref();
        let max_width = layout.line_width() as usize;

        if max_width <= 3 {
            return Err(std::fmt::Error.into());
        }

        if s.chars().count() > max_width {
            let byte_i = s.char_indices().nth(max_width - 3).unwrap().0;
            backend.write_all(s[..byte_i].as_bytes())?;
            backend.write_all(b"...").map_err(Into::into)
        } else {
            backend.write_all(s.as_bytes()).map_err(Into::into)
        }
    }

    /// Does not allow multi-line strings
    fn height(&mut self, _: Layout) -> u16 {
        0
    }
}

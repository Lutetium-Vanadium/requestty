use std::{fmt, io::Write};

use crossterm::event;

/// A trait to represent renderable objects.
pub trait Widget {
    /// Handle a key input. It should return whether key was handled.
    #[allow(unused_variables)]
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        false
    }

    /// Render to stdout. `max_width` is the number of characters that can be printed in the current
    /// line.
    fn render<W: Write>(&mut self, max_width: usize, stdout: &mut W) -> crossterm::Result<()>;

    /// The number of rows of the terminal the widget will take when rendered
    fn height(&self) -> usize;

    /// The position of the cursor to end at, with (0,0) being the start of the input
    #[allow(unused_variables)]
    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        (prompt_len, 0)
    }
}

impl<T: AsRef<str>> Widget for T {
    fn render<W: Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        let s = self.as_ref();

        if max_width <= 3 {
            return Err(fmt::Error.into());
        }

        if s.chars().count() > max_width {
            let byte_i = s.char_indices().nth(max_width - 3).unwrap().0;
            w.write_all(s[..byte_i].as_bytes())?;
            w.write_all(b"...").map_err(Into::into)
        } else {
            w.write_all(s.as_bytes()).map_err(Into::into)
        }
    }

    fn height(&self) -> usize {
        0
    }
}

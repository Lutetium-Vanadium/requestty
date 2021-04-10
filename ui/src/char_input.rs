use std::{fmt, io::Write};

use crossterm::event;

use crate::widget::Widget;

/// A widget that inputs a single character. If multiple characters are inputted to it, it will have
/// the last character
pub struct CharInput<F = crate::widgets::FilterMapChar> {
    value: Option<char>,
    filter_map_char: F,
}

impl<F> CharInput<F>
where
    F: Fn(char) -> Option<char>,
{
    /// Creates a new [`CharInput`]. The filter_map_char is used in [`CharInput::handle_key`] to
    /// avoid some characters to limit and filter characters.
    pub fn new(filter_map_char: F) -> Self {
        Self {
            value: None,
            filter_map_char,
        }
    }

    /// The last inputted char (if any)
    pub fn value(&self) -> Option<char> {
        self.value
    }

    /// Set the value
    pub fn set_value(&mut self, value: Option<char>) {
        self.value = value;
    }

    /// Consumes self, returning the value
    pub fn finish(self) -> Option<char> {
        self.value
    }
}

impl<F> Widget for CharInput<F>
where
    F: Fn(char) -> Option<char>,
{
    /// Handles character, backspace and delete events.
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        match key.code {
            event::KeyCode::Char(c) => {
                if let Some(c) = (self.filter_map_char)(c) {
                    self.value = Some(c);

                    return true;
                }

                false
            }

            event::KeyCode::Backspace | event::KeyCode::Delete if self.value.is_some() => {
                self.value = None;
                true
            }

            _ => false,
        }
    }

    fn render<W: Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        if let Some(value) = self.value {
            if max_width == 0 {
                return Err(fmt::Error.into());
            }

            write!(w, "{}", value)?;
        }
        Ok(())
    }

    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        (self.value.map(|_| 1).unwrap_or(0) + prompt_len, 0)
    }

    fn height(&self) -> usize {
        0
    }
}

impl Default for CharInput {
    fn default() -> Self {
        Self::new(crate::widgets::no_filter)
    }
}

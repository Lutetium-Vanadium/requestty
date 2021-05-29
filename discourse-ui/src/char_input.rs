use crate::{
    backend::Backend,
    error,
    events::{KeyCode, KeyEvent},
    Layout,
};

/// A widget that inputs a single character. If multiple characters are inputted to it, it will have
/// the last character
pub struct CharInput<F = super::widgets::FilterMapChar> {
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

impl<F> super::Widget for CharInput<F>
where
    F: Fn(char) -> Option<char>,
{
    /// Handles character, backspace and delete events.
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                if let Some(c) = (self.filter_map_char)(c) {
                    self.value = Some(c);

                    return true;
                }

                false
            }

            KeyCode::Backspace | KeyCode::Delete if self.value.is_some() => {
                self.value = None;
                true
            }

            _ => false,
        }
    }

    fn render<B: Backend>(
        &mut self,
        layout: Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        if let Some(value) = self.value {
            if layout.line_width() == 0 {
                return Err(std::fmt::Error.into());
            }

            write!(backend, "{}", value)?;
        }
        Ok(())
    }

    fn height(&mut self, _: Layout) -> u16 {
        1
    }

    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        (self.value.map(|_| 1).unwrap_or(0) + layout.line_offset, 0)
    }
}

impl Default for CharInput {
    fn default() -> Self {
        Self::new(super::widgets::no_filter)
    }
}

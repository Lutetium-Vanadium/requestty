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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::{TestBackend, TestBackendOp::Write},
        events::KeyModifiers,
        widgets::no_filter,
        Widget,
    };

    #[test]
    fn test_cursor_pos() {
        let layout = Layout::new(0, (100, 40).into());
        let mut input = CharInput::new(no_filter);

        assert_eq!(input.cursor_pos(layout), (0, 0));
        assert_eq!(input.cursor_pos(layout.with_line_offset(5)), (5, 0));

        input.set_value(Some('c'));

        assert_eq!(input.cursor_pos(layout), (1, 0));
        assert_eq!(input.cursor_pos(layout.with_line_offset(5)), (6, 0));
    }

    #[test]
    fn test_handle_key() {
        let modifiers = KeyModifiers::empty();

        let mut input = CharInput::new(crate::widgets::no_filter);
        assert!(input.handle_key(KeyEvent::new(KeyCode::Char('c'), modifiers)));
        assert_eq!(input.value(), Some('c'));
        assert!(!input.handle_key(KeyEvent::new(KeyCode::Tab, modifiers)));
        assert!(input.handle_key(KeyEvent::new(KeyCode::Char('d'), modifiers)));
        assert_eq!(input.value(), Some('d'));
        assert!(input.handle_key(KeyEvent::new(KeyCode::Backspace, modifiers)));
        assert_eq!(input.value(), None);
        assert!(input.handle_key(KeyEvent::new(KeyCode::Char('c'), modifiers)));
        assert_eq!(input.value(), Some('c'));
        assert!(input.handle_key(KeyEvent::new(KeyCode::Delete, modifiers)));
        assert_eq!(input.value(), None);
        assert!(!input.handle_key(KeyEvent::new(KeyCode::Delete, modifiers)));
        assert!(!input.handle_key(KeyEvent::new(KeyCode::Backspace, modifiers)));

        let mut input =
            CharInput::new(|c| if c.is_uppercase() { None } else { Some(c) });
        assert!(!input.handle_key(KeyEvent::new(KeyCode::Char('C'), modifiers)));
        assert_eq!(input.value(), None);
        assert!(input.handle_key(KeyEvent::new(KeyCode::Char('c'), modifiers)));
        assert_eq!(input.value(), Some('c'));
        assert!(!input.handle_key(KeyEvent::new(KeyCode::Char('C'), modifiers)));
        assert_eq!(input.value(), Some('c'));
    }

    #[test]
    fn test_render() {
        let size = (100, 40).into();
        let layout = Layout::new(0, size);

        CharInput::new(no_filter)
            .render(layout, &mut TestBackend::new(None, size))
            .unwrap();

        let mut input = CharInput::new(no_filter);
        input.set_value(Some('c'));

        input
            .render(
                layout,
                &mut TestBackend::new(Some(Write(b"c".to_vec())), size),
            )
            .unwrap();
    }
}

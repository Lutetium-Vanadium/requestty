use std::{
    io::{self, Write},
    ops::Range,
};

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    backend::Backend,
    error,
    events::{KeyCode, KeyEvent, KeyModifiers, Movement},
    layout::Layout,
};

/// A widget that inputs a line of text
pub struct StringInput<F = super::widgets::FilterMapChar> {
    value: String,
    mask: Option<char>,
    hide_output: bool,
    /// The character length of the string
    value_len: usize,
    /// The position of the 'cursor' in characters
    at: usize,
    filter_map_char: F,
}

impl<F> StringInput<F> {
    /// Creates a new [`StringInput`]. The filter_map_char is used in [`StringInput::handle_key`] to
    /// avoid some characters to limit and filter characters.
    pub fn new(filter_map_char: F) -> Self {
        Self {
            value: String::new(),
            value_len: 0,
            at: 0,
            filter_map_char,
            mask: None,
            hide_output: false,
        }
    }

    /// A mask to print in render instead of the actual characters
    pub fn mask(mut self, mask: char) -> Self {
        self.mask = Some(mask);
        self
    }

    /// Whether to render nothing, but still keep track of all the characters
    pub fn hide_output(mut self) -> Self {
        self.hide_output = true;
        self
    }

    /// A helper that sets mask if mask is some, otherwise hides the output
    pub fn password(self, mask: Option<char>) -> Self {
        match mask {
            Some(mask) => self.mask(mask),
            None => self.hide_output(),
        }
    }

    pub fn get_at(&self) -> usize {
        self.at
    }

    pub fn set_at(&mut self, at: usize) {
        self.at = at.min(self.value_len);
    }

    /// The currently inputted value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the value
    pub fn set_value(&mut self, value: String) {
        self.value_len = value.chars().count();
        self.value = value;
        self.set_at(self.at);
    }

    /// Replaces the value with the result of the function
    pub fn replace_with<W: FnOnce(String) -> String>(&mut self, with: W) {
        self.value = with(std::mem::take(&mut self.value));
        let old_len = self.value_len;
        self.value_len = self.value.chars().count();
        if self.at == old_len {
            self.at = self.value_len;
        } else {
            self.set_at(self.at);
        }
    }

    /// Check whether any character has come to the input
    pub fn has_value(&self) -> bool {
        self.value.capacity() > 0
    }

    /// Returns None if no characters have been inputted, otherwise returns Some
    ///
    /// note: it can return Some(""), if a character was added and then deleted. It will only return
    /// None when no character was ever received
    pub fn finish(self) -> Option<String> {
        if self.has_value() {
            Some(self.value)
        } else {
            None
        }
    }

    /// Gets the byte index of a given char index
    fn get_byte_i(&self, index: usize) -> usize {
        self.value
            .char_indices()
            .nth(index)
            .map(|(i, _)| i)
            .unwrap_or_else(|| self.value.len())
    }

    /// Gets the char index of a given byte index
    fn get_char_i(&self, byte_i: usize) -> usize {
        self.value
            .char_indices()
            .position(|(i, _)| i == byte_i)
            .unwrap_or_else(|| self.value.char_indices().count())
    }

    /// Get the word bound iterator for a given range
    fn word_iter(
        &self,
        r: Range<usize>,
    ) -> impl DoubleEndedIterator<Item = (usize, &str)> {
        self.value[r].split_word_bound_indices().filter(|(_, s)| {
            !s.chars().next().map(char::is_whitespace).unwrap_or(true)
        })
    }

    /// Returns the byte index of the start of the first word to the left (< byte_i)
    fn find_word_left(&self, byte_i: usize) -> usize {
        self.word_iter(0..byte_i)
            .next_back()
            .map(|(new_byte_i, _)| new_byte_i)
            .unwrap_or(0)
    }

    /// Returns the byte index of the start of the first word to the right (> byte_i)
    fn find_word_right(&self, byte_i: usize) -> usize {
        self.word_iter(byte_i..self.value.len())
            .nth(1)
            .map(|(new_byte_i, _)| new_byte_i + byte_i)
            .unwrap_or_else(|| self.value.len())
    }

    fn get_delete_movement(&self, key: KeyEvent) -> Option<Movement> {
        let mov = match key.code {
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::Home
            }
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
                Movement::PrevWord
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::ALT) => {
                Movement::PrevWord
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::Left
            }
            KeyCode::Backspace => Movement::Left,

            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::End
            }

            KeyCode::Delete if key.modifiers.contains(KeyModifiers::ALT) => {
                Movement::NextWord
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
                Movement::NextWord
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::Right
            }
            KeyCode::Delete => Movement::Right,

            _ => return None,
        };

        match mov {
            Movement::Home | Movement::PrevWord | Movement::Left if self.at != 0 => {
                Some(mov)
            }
            Movement::End | Movement::NextWord | Movement::Right
                if self.at != self.value_len =>
            {
                Some(mov)
            }
            _ => None,
        }
    }
}

impl<F> super::Widget for StringInput<F>
where
    F: Fn(char) -> Option<char>,
{
    /// Handles characters, backspace, delete, left arrow, right arrow, home and end.
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if let Some(movement) = self.get_delete_movement(key) {
            match movement {
                Movement::Home => {
                    let byte_i = self.get_byte_i(self.at);
                    self.value_len -= self.at;
                    self.at = 0;
                    self.value.replace_range(..byte_i, "");
                    return true;
                }
                Movement::PrevWord => {
                    let was_at = self.at;
                    let byte_i = self.get_byte_i(self.at);
                    let prev_word = self.find_word_left(byte_i);
                    self.at = self.get_char_i(prev_word);
                    self.value_len -= was_at - self.at;
                    self.value.replace_range(prev_word..byte_i, "");
                    return true;
                }
                Movement::Left if self.at == self.value_len => {
                    self.at -= 1;
                    self.value_len -= 1;
                    self.value.pop();
                    return true;
                }
                Movement::Left => {
                    self.at -= 1;
                    let byte_i = self.get_byte_i(self.at);
                    self.value_len -= 1;
                    self.value.remove(byte_i);
                    return true;
                }

                Movement::End => {
                    let byte_i = self.get_byte_i(self.at);
                    self.value_len = self.at;
                    self.value.truncate(byte_i);
                    return true;
                }
                Movement::NextWord => {
                    let byte_i = self.get_byte_i(self.at);
                    let next_word = self.find_word_right(byte_i);
                    self.value_len -= self.get_char_i(next_word) - self.at;
                    self.value.replace_range(byte_i..next_word, "");
                    return true;
                }
                Movement::Right if self.at == self.value_len - 1 => {
                    self.value_len -= 1;
                    self.value.pop();
                    return true;
                }
                Movement::Right => {
                    let byte_i = self.get_byte_i(self.at);
                    self.value_len -= 1;
                    self.value.remove(byte_i);
                    return true;
                }

                _ => {}
            }
        }

        match key.code {
            // FIXME: all chars with ctrl and alt are ignored, even though only some
            // need to be ignored
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(c) = (self.filter_map_char)(c) {
                    if self.at == self.value_len {
                        self.value.push(c);
                    } else {
                        let byte_i = self.get_byte_i(self.at);
                        self.value.insert(byte_i, c);
                    };

                    self.at += 1;
                    self.value_len += 1;
                    return true;
                }
            }

            _ => {}
        }

        match Movement::try_from_key(key) {
            Some(Movement::PrevWord) if self.at != 0 => {
                self.at =
                    self.get_char_i(self.find_word_left(self.get_byte_i(self.at)));
            }
            Some(Movement::Left) if self.at != 0 => {
                self.at -= 1;
            }

            Some(Movement::NextWord) if self.at != self.value_len => {
                self.at =
                    self.get_char_i(self.find_word_right(self.get_byte_i(self.at)));
            }
            Some(Movement::Right) if self.at != self.value_len => {
                self.at += 1;
            }

            Some(Movement::Home) if self.at != 0 => {
                self.at = 0;
            }
            Some(Movement::End) if self.at != self.value_len => {
                self.at = self.value_len;
            }
            _ => return false,
        }

        true
    }

    fn render<B: Backend>(
        &mut self,
        layout: &mut Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        if self.hide_output {
            return Ok(());
        }

        if let Some(mask) = self.mask {
            print_mask(self.value_len, mask, backend)?;
        } else {
            // Terminal takes care of wrapping in case of large strings
            backend.write_all(self.value.as_bytes())?;
        }

        // Adjust layout
        self.height(layout);

        Ok(())
    }

    fn height(&mut self, layout: &mut Layout) -> u16 {
        if self.hide_output {
            return 1;
        }

        let mut width = self.value_len as u16;
        if width > layout.line_width() {
            width -= layout.line_width();

            layout.line_offset = width % layout.width;
            layout.offset_y += 1 + width / layout.width;

            2 + width / layout.width
        } else {
            layout.line_offset += width;
            1
        }
    }

    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        if self.hide_output {
            // Nothing will be outputted so no need to move the cursor
            (layout.line_offset, 0)
        } else if layout.line_width() > self.at as u16 {
            // It is in the same line as the prompt
            (layout.line_offset + self.at as u16, 0)
        } else {
            let at = self.at as u16 - layout.line_width();

            (at % layout.width, 1 + at / layout.width)
        }
    }
}

impl Default for StringInput {
    fn default() -> Self {
        Self::new(super::widgets::no_filter)
    }
}

fn print_mask<W: Write>(len: usize, mask: char, w: &mut W) -> io::Result<()> {
    let mut buf = [0; 4];
    let mask = mask.encode_utf8(&mut buf[..]);

    for _ in 0..len {
        w.write_all(mask.as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::{TestBackend, TestBackendOp::Write},
        events::KeyModifiers,
        test_consts::*,
        Widget,
    };

    #[test]
    fn test_print_mask() {
        fn test(mask: char) {
            let mut buf = [0u8; 100];
            let mask_len = mask.len_utf8();
            print_mask(25, mask, &mut &mut buf[..]).unwrap();
            assert!(std::str::from_utf8(&buf[0..(25 * mask_len)])
                .unwrap()
                .chars()
                .all(|c| c == mask));
            assert!(buf[(25 * mask_len)..].iter().all(|&b| b == 0));
        }

        test('*');
        test('‣');
    }

    #[test]
    fn test_delete_movement() {
        let mut input = StringInput::default();

        let backspace_movements = [
            (
                KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                Movement::Home,
            ),
            (
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT),
                Movement::PrevWord,
            ),
            (
                KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT),
                Movement::PrevWord,
            ),
            (
                KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
                Movement::Left,
            ),
            (
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()),
                Movement::Left,
            ),
        ];

        let delete_movements = [
            (
                KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL),
                Movement::End,
            ),
            (
                KeyEvent::new(KeyCode::Delete, KeyModifiers::ALT),
                Movement::NextWord,
            ),
            (
                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT),
                Movement::NextWord,
            ),
            (
                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                Movement::Right,
            ),
            (
                KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()),
                Movement::Right,
            ),
        ];

        assert!(backspace_movements
            .iter()
            .copied()
            .all(|(key, _)| input.get_delete_movement(key).is_none()));

        assert!(delete_movements
            .iter()
            .copied()
            .all(|(key, _)| input.get_delete_movement(key).is_none()));

        input.set_value("Hello world".into());

        assert!(backspace_movements
            .iter()
            .copied()
            .all(|(key, _)| input.get_delete_movement(key).is_none()));
        assert!(delete_movements
            .iter()
            .copied()
            .all(|(key, mov)| input.get_delete_movement(key).unwrap() == mov));

        input.set_at(11);

        assert!(backspace_movements
            .iter()
            .copied()
            .all(|(key, mov)| input.get_delete_movement(key).unwrap() == mov));
        assert!(delete_movements
            .iter()
            .copied()
            .all(|(key, _)| input.get_delete_movement(key).is_none()));

        input.set_at(5);

        assert!(backspace_movements
            .iter()
            .copied()
            .all(|(key, mov)| input.get_delete_movement(key).unwrap() == mov));
        assert!(delete_movements
            .iter()
            .copied()
            .all(|(key, mov)| input.get_delete_movement(key).unwrap() == mov));
    }

    #[test]
    fn test_render() {
        fn test(text: &str, line_offset: u16, offset_y: u16) {
            let size = (100, 40).into();
            let mut layout = Layout::new(0, size);

            let mut input = StringInput::default();
            input.set_value(text.into());
            input
                .render(
                    &mut layout,
                    &mut TestBackend::new(Some(Write(text.into())), size),
                )
                .unwrap();

            assert_eq!(
                layout,
                Layout::new(0, size)
                    .with_line_offset(line_offset)
                    .with_offset(0, offset_y)
            );
        }

        test("Hello, World!", 13, 0);
        test(LOREM, 70, 4);
        test(UNICODE, 70, 4);
    }

    #[test]
    fn test_handle_key() {
        let mut input = StringInput::new(|c| if c == 'i' { None } else { Some(c) });
        input.set_value(LOREM.into());
        input.set_at(40);

        input.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 39);
        input.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 28);
        input.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 29);
        input.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 41);
        input.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 0);
        input.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 470);

        input.set_at(40);
        input.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 39);
        assert_eq!(input.value().chars().count(), 469);
        input.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty()));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value(), LOREM);

        input.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 0);
        assert_eq!(input.value().chars().count(), 430);
        input.set_at(400);
        input.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 400);
        assert_eq!(input.value().chars().count(), 400);
        input.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT));
        assert_eq!(input.get_at(), 394);
        assert_eq!(input.value().chars().count(), 394);

        input.set_at(40);
        input.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 393);
        input.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::ALT));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 388);

        input.handle_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 388);
        input.handle_key(KeyEvent::new(KeyCode::Char('I'), KeyModifiers::empty()));
        assert_eq!(input.get_at(), 41);
        assert_eq!(input.value().chars().count(), 389);

        let mut input = StringInput::new(|c| if c == 'ȼ' { None } else { Some(c) });
        input.set_value(UNICODE.into());
        input.set_at(40);

        input.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 39);
        input.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 31);
        input.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 32);
        input.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 39);
        input.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 0);
        input.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 470);

        input.set_at(41);
        input.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 469);
        input.handle_key(KeyEvent::new(KeyCode::Char('Æ'), KeyModifiers::empty()));
        assert_eq!(input.get_at(), 41);
        assert_eq!(input.value(), UNICODE);
        input.set_at(40);

        input.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 0);
        assert_eq!(input.value().chars().count(), 430);
        input.set_at(400);
        input.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
        assert_eq!(input.get_at(), 400);
        assert_eq!(input.value().chars().count(), 400);
        input.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::ALT));
        assert_eq!(input.get_at(), 397);
        assert_eq!(input.value().chars().count(), 397);

        input.set_at(40);
        input.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::empty()));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 396);
        input.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::ALT));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 386);

        input.handle_key(KeyEvent::new(KeyCode::Char('ȼ'), KeyModifiers::empty()));
        assert_eq!(input.get_at(), 40);
        assert_eq!(input.value().chars().count(), 386);
    }

    #[test]
    fn test_height() {
        fn test(text: &str, indent: usize, max_width: usize, height: u16) {
            let mut layout =
                Layout::new(indent as u16, (max_width as u16, 100).into());
            let mut input = StringInput::default();
            input.set_value(text.into());
            assert_eq!(input.height(&mut layout), height);
        }

        test("Hello, World!", 0, 100, 1);
        test("Hello, World!", 0, 7, 2);
        test("Hello, World!", 2, 7, 3);

        test(LOREM, 0, 100, 5);
        test(LOREM, 40, 100, 6);
    }

    #[test]
    fn test_cursor_pos() {
        let mut layout = Layout::new(0, (100, 100).into());
        let mut input = StringInput::default();
        input.set_value("Hello, World!".into());
        input.set_at(0);
        assert_eq!(input.cursor_pos(layout), (0, 0));
        input.set_at(4);
        assert_eq!(input.cursor_pos(layout), (4, 0));

        layout.line_offset = 5;
        assert_eq!(input.cursor_pos(layout), (9, 0));
        input.set_at(0);
        assert_eq!(input.cursor_pos(layout), (5, 0));

        layout.line_offset = 0;
        input.set_value(LOREM.into());
        assert_eq!(input.cursor_pos(layout), (0, 0));
        input.set_at(4);
        assert_eq!(input.cursor_pos(layout), (4, 0));

        layout.line_offset = 5;
        assert_eq!(input.cursor_pos(layout), (9, 0));
        input.set_at(0);
        assert_eq!(input.cursor_pos(layout), (5, 0));

        input.set_at(130);
        assert_eq!(input.cursor_pos(layout), (35, 1));

        layout.line_offset = 0;
        input.set_at(0);
        input.set_value(UNICODE.into());
        assert_eq!(input.cursor_pos(layout), (0, 0));
        input.set_at(4);
        assert_eq!(input.cursor_pos(layout), (4, 0));

        layout.line_offset = 5;
        assert_eq!(input.cursor_pos(layout), (9, 0));
        input.set_at(0);
        assert_eq!(input.cursor_pos(layout), (5, 0));

        input.set_at(130);
        assert_eq!(input.cursor_pos(layout), (35, 1));
    }
}

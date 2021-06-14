use crate::{backend, error, layout::Layout, Widget};

#[derive(Debug)]
pub struct Text<S> {
    pub text: S,
    wrapped: String,
    line_offset: u16,
    width: u16,
}

impl<S: AsRef<str>> Text<S> {
    pub fn new(text: S) -> Self {
        Self {
            text,
            wrapped: String::new(),
            width: 0,
            line_offset: 0,
        }
    }

    fn max_height(&mut self, layout: Layout) -> u16 {
        let width = layout.available_width();

        if self.width != width || self.line_offset != layout.line_offset {
            self.wrapped = fill(self.text.as_ref(), layout);
            self.width = width;
            self.line_offset = layout.line_offset;
        }

        self.wrapped.lines().count() as u16
    }
}

impl<S: AsRef<str>> Widget for Text<S> {
    fn render<B: backend::Backend>(
        &mut self,
        layout: &mut Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        if layout.max_height == 0 {
            return Err(std::fmt::Error.into());
        }

        // Update just in case the layout is out of date
        let height = self.max_height(*layout);

        if height == 1 {
            backend.write_all(self.wrapped.as_bytes())?;
            layout.offset_y += 1;
            backend.move_cursor_to(layout.offset_x, layout.offset_y)?;
        } else {
            let start = layout.get_start(height) as usize;
            let nlines = height.min(layout.max_height);

            for (i, line) in self
                .wrapped
                .lines()
                .skip(start)
                .take(nlines as usize)
                .enumerate()
            {
                backend.write_all(line.as_bytes())?;
                backend.move_cursor_to(
                    layout.offset_x,
                    layout.offset_y + i as u16 + 1,
                )?;
            }

            // note: it may be possible to render things after the end of the last line, but for now
            // we ignore that space and the text takes all the width.
            layout.offset_y += nlines;
        }
        layout.line_offset = 0;

        Ok(())
    }

    fn height(&mut self, layout: &mut Layout) -> u16 {
        let height = self.max_height(*layout).min(layout.max_height);
        layout.offset_y += height;
        height
    }

    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        self.text.as_ref().cursor_pos(layout)
    }

    fn handle_key(&mut self, key: crate::events::KeyEvent) -> bool {
        self.text.as_ref().handle_key(key)
    }
}

impl<S: AsRef<str>> AsRef<str> for Text<S> {
    fn as_ref(&self) -> &str {
        self.text.as_ref()
    }
}

// 200 spaces to remove allocation for indent
static SPACES: &str = "                                                                                                                                                                                                        ";

fn fill(text: &str, layout: Layout) -> String {
    // This won't allocate until the **highly unlikely** case that there is a line
    // offset of more than 200.
    let s: String;

    let indent_len = layout.line_offset as usize;

    let indent = if SPACES.len() > indent_len {
        &SPACES[..indent_len]
    } else {
        s = " ".repeat(indent_len);
        &s[..]
    };

    let mut text = textwrap::fill(
        text,
        textwrap::Options::new(layout.available_width() as usize)
            .initial_indent(indent),
    );

    drop(text.drain(..indent_len));

    text
}

impl<E: std::error::Error> From<E> for Text<String> {
    fn from(e: E) -> Self {
        Text::new(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::{TestBackend, TestBackendOp::*},
        test_consts::*,
    };

    use super::*;

    #[test]
    fn test_fill() {
        fn test(text: &str, indent: usize, max_width: usize, nlines: usize) {
            let layout = Layout::new(indent as u16, (max_width as u16, 100).into());
            let filled = fill(text, layout);

            assert_eq!(nlines, filled.lines().count());
            let mut lines = filled.lines();

            assert!(lines.next().unwrap().chars().count() <= max_width - indent);

            for line in lines {
                assert!(line.chars().count() <= max_width);
            }
        }

        test("Hello World", 0, 80, 1);

        test("Hello World", 0, 6, 2);

        test(LOREM, 40, 80, 7);
        test(UNICODE, 40, 80, 7);
    }

    #[test]
    fn test_text_height() {
        let mut layout = Layout::new(40, (80, 100).into());
        let mut text = Text::new(LOREM);

        assert_eq!(text.max_height(layout), 7);
        assert_eq!(text.height(&mut layout.with_max_height(5)), 5);
        layout.line_offset = 0;
        layout.width = 110;
        assert_eq!(text.height(&mut layout.clone()), text.max_height(layout));
        assert_eq!(text.height(&mut layout.clone()), 5);

        let mut layout = Layout::new(40, (80, 100).into());
        let mut text = Text::new(UNICODE);

        assert_eq!(text.max_height(layout), 7);
        assert_eq!(text.height(&mut layout.with_max_height(5)), 5);
        layout.line_offset = 0;
        layout.width = 110;
        assert_eq!(text.height(&mut layout.clone()), text.max_height(layout));
        assert_eq!(text.height(&mut layout.clone()), 5);
    }

    #[test]
    fn test_render_single_line() {
        let size = (100, 100).into();
        let mut layout = Layout::new(0, size);
        let mut backend = TestBackend::new(
            vec![Write(b"Hello, World!".to_vec()), MoveCursorTo(0, 1)],
            size,
        );
        let mut text = Text::new("Hello, World!");
        text.render(&mut layout, &mut backend).unwrap();
        assert_eq!(layout, layout.with_offset(0, 1));
    }

    #[test]
    fn test_render_multiline() {
        let size = (100, 100).into();
        let mut layout = Layout::new(0, size);

        let mut ops = Vec::with_capacity(10);
        for (i, line) in fill(LOREM, layout).lines().enumerate() {
            ops.push(Write(line.into()));
            ops.push(MoveCursorTo(0, i as u16 + 1));
        }

        let mut backend = TestBackend::new(ops, size);
        let mut text = Text::new(LOREM);
        text.render(&mut layout, &mut backend).unwrap();

        assert_eq!(layout, Layout::new(0, size).with_offset(0, 5));
        layout = Layout::new(0, size).with_offset(10, 10);

        let mut ops = Vec::with_capacity(10);
        for (i, line) in fill(UNICODE, layout).lines().enumerate() {
            ops.push(Write(line.into()));
            ops.push(MoveCursorTo(10, i as u16 + 11));
        }

        let mut backend = TestBackend::new(ops, size);
        let mut text = Text::new(UNICODE);
        text.render(&mut layout, &mut backend).unwrap();

        assert_eq!(layout, Layout::new(0, size).with_offset(10, 16));
    }
}

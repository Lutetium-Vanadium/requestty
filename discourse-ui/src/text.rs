use crate::{backend, error, layout::Layout, Widget};

/// A string that can render over multiple lines.
///
/// If you need to render a single line of text or you don't want the text to wrap, use the [`Widget`]
/// implementation on [`str`].
#[derive(Debug, Clone)]
pub struct Text<S> {
    /// The text to render.
    ///
    /// If this is changed, the updated text is not guaranteed to be rendered. If the text is
    /// changed, [`force_recompute`](Text::force_recompute) should be called.
    pub text: S,
    // FIXME: currently textwrap doesn't provide a way to find the locations at which the text
    // should be split. Using that will be much more efficient than essentially duplicating the
    // string.
    wrapped: String,
    line_offset: u16,
    width: u16,
}

impl<S: PartialEq> PartialEq for Text<S> {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl<S: Eq> Eq for Text<S> {}

impl<S: AsRef<str>> Text<S> {
    /// Creates a new `Text`
    pub fn new(text: S) -> Self {
        Self {
            text,
            wrapped: String::new(),
            width: 0,
            line_offset: 0,
        }
    }

    /// The computed lines are cached between renders, and are only recomputed if the layout changes.
    /// This will force a recomputation even if the layout is the same. This is useful if you need
    /// to change the text.
    pub fn force_recompute(&mut self) {
        self.line_offset = u16::MAX;
        self.width = u16::MAX;
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
    /// Renders the Text moving to the next line after its done. This can trigger a recomputation.
    /// In case the text cannot be fully rendered, [`layout.render_region`] is used to determine the
    /// lines which are rendered.
    ///
    /// [`layout.render_region`]: crate::layout::Layout::render_region
    fn render<B: backend::Backend>(
        &mut self,
        layout: &mut Layout,
        backend: &mut B,
    ) -> error::Result<()> {
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
                backend.move_cursor_to(layout.offset_x, layout.offset_y + i as u16 + 1)?;
            }

            // note: it may be possible to render things after the end of the last line, but for now
            // we ignore that space and the text takes all the width.
            layout.offset_y += nlines;
        }
        layout.line_offset = 0;

        Ok(())
    }

    /// Calculates the height the text will take. This can trigger a recomputation.
    fn height(&mut self, layout: &mut Layout) -> u16 {
        let height = self.max_height(*layout).min(layout.max_height);
        layout.offset_y += height;
        height
    }

    /// Returns the location of the first character
    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        (layout.line_offset, 0)
    }

    /// This widget does not handle any events
    fn handle_key(&mut self, _: crate::events::KeyEvent) -> bool {
        false
    }
}

impl<S: AsRef<str>> AsRef<str> for Text<S> {
    fn as_ref(&self) -> &str {
        self.text.as_ref()
    }
}

impl<S: AsRef<str>> From<S> for Text<S> {
    fn from(text: S) -> Self {
        Self::new(text)
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
        textwrap::Options::new(layout.available_width() as usize).initial_indent(indent),
    );

    drop(text.drain(..indent_len));

    text
}

#[cfg(test)]
mod tests {
    use crate::{backend::TestBackend, test_consts::*};

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
        let size = (100, 20).into();
        let mut layout = Layout::new(0, size);
        let mut backend = TestBackend::new(size);

        let mut text = Text::new("Hello, World!");
        text.render(&mut layout, &mut backend).unwrap();

        crate::assert_backend_snapshot!(backend);
        assert_eq!(layout, layout.with_offset(0, 1));
    }

    #[test]
    fn test_render_multiline() {
        let size = (100, 20).into();
        let mut layout = Layout::new(0, size);

        let mut backend = TestBackend::new(size);
        let mut text = Text::new(LOREM);
        text.render(&mut layout, &mut backend).unwrap();

        crate::assert_backend_snapshot!(backend);
        assert_eq!(layout, Layout::new(0, size).with_offset(0, 5));

        layout = Layout::new(0, size).with_offset(10, 10);
        backend.reset_with_layout(layout);

        let mut text = Text::new(UNICODE);
        text.render(&mut layout, &mut backend).unwrap();

        crate::assert_backend_snapshot!(backend);
        assert_eq!(layout, Layout::new(0, size).with_offset(10, 16));
    }
}

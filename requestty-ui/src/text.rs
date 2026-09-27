use std::ops::Range;

use textwrap::{
    core::{break_words, Word},
    word_splitters::split_words,
    WordSeparator, WordSplitter, WrapAlgorithm,
};

use crate::{backend, layout::Layout, Widget};

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
    /// Byte ranges into `text` for each wrapped line.
    lines: Vec<Range<usize>>,
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
            lines: Vec::new(),
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
            wrap(self.text.as_ref(), layout, &mut self.lines);
            self.width = width;
            self.line_offset = layout.line_offset;
        }

        self.lines.len() as u16
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
    ) -> std::io::Result<()> {
        // Update just in case the layout is out of date
        let height = self.max_height(*layout);
        let text = self.text.as_ref();

        if height == 1 {
            backend.write_all(text[self.lines[0].clone()].as_bytes())?;
            layout.offset_y += 1;
            backend.move_cursor_to(layout.offset_x, layout.offset_y)?;
        } else {
            let start = layout.get_start(height) as usize;
            let nlines = height.min(layout.max_height);

            for (i, line) in self
                .lines
                .iter()
                .skip(start)
                .take(nlines as usize)
                .enumerate()
            {
                backend.write_all(text[line.clone()].as_bytes())?;
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
        layout.offset_cursor((layout.line_offset, 0))
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

/// Wraps `text` into lines, storing the byte range of each line in `lines`.
///
/// This mirrors what `textwrap::fill` does with an initial indent of `layout.line_offset`, but
/// borrows from `text` instead of building a new string.
fn wrap(text: &str, layout: Layout, lines: &mut Vec<Range<usize>>) {
    lines.clear();

    let width = layout.available_width() as usize;
    let line_widths = [width.saturating_sub(layout.line_offset as usize), width];

    let mut start = 0;
    for line in text.split_inclusive('\n') {
        let next_start = start + line.len();
        let line = line.strip_suffix('\n').unwrap_or(line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let first_line = lines.is_empty();

        if line.len() < width && !(first_line && layout.line_offset > 0) {
            // Fast path: the line fits, so only trailing spaces need to be removed.
            lines.push(start..start + line.trim_end_matches(' ').len());
        } else {
            wrap_line(line, start, &line_widths, first_line, lines);
        }

        start = next_start;
    }
}

fn wrap_line(
    line: &str,
    start: usize,
    line_widths: &[usize; 2],
    first_line: bool,
    lines: &mut Vec<Range<usize>>,
) {
    let words = WordSeparator::new().find_words(line);
    let words = split_words(words, &WordSplitter::HyphenSplitter);
    let mut words = break_words(words, line_widths[1]);

    if first_line && line_widths[0] != line_widths[1] {
        // Words are broken based on the width of the subsequent lines, so the first word may not
        // fit on the (shorter) first line. An empty word allows the first line to be empty.
        words.insert(0, Word::from(""));
    }

    let mut idx = start;
    for words in WrapAlgorithm::new().wrap(&words, line_widths) {
        let last_word = match words.last() {
            Some(word) => word,
            None => {
                lines.push(idx..idx);
                continue;
            }
        };

        // The hyphen splitter only splits after existing hyphens, so no extra characters are
        // added and every line is a contiguous slice of the original text.
        debug_assert!(last_word.penalty.is_empty());

        let len = words
            .iter()
            .map(|word| word.len() + word.whitespace.len())
            .sum::<usize>();

        lines.push(idx..idx + len - last_word.whitespace.len());
        idx += len;
    }
}

#[cfg(test)]
mod tests {
    use crate::{backend::TestBackend, test_consts::*};

    use super::*;

    #[test]
    fn test_wrap() {
        fn test(text: &str, indent: usize, max_width: usize, nlines: usize) {
            let layout = Layout::new(indent as u16, (max_width as u16, 100).into());
            let mut wrapped = Vec::new();
            wrap(text, layout, &mut wrapped);

            // Should match the output of `textwrap::fill`
            let indent_str = " ".repeat(indent);
            let filled = textwrap::fill(
                text,
                textwrap::Options::new(max_width).initial_indent(&indent_str),
            );
            let expected: Vec<_> = filled[indent..].lines().collect();
            let actual: Vec<_> = wrapped.iter().map(|r| &text[r.clone()]).collect();
            assert_eq!(expected, actual);

            assert_eq!(nlines, wrapped.len());
            let mut lines = actual.into_iter();

            assert!(lines.next().unwrap().chars().count() <= max_width - indent);

            for line in lines {
                assert!(line.chars().count() <= max_width);
            }
        }

        test("Hello World", 0, 80, 1);

        test("Hello World", 0, 6, 2);

        test(LOREM, 40, 80, 7);
        test(UNICODE, 40, 80, 7);

        test("Hello\n\nWorld  \n", 0, 80, 3);
        test("Hello\r\nWorld", 3, 80, 2);
        test(
            "a-very-long-hyphenated-word and a supercalifragilisticexpialidocious one",
            5,
            12,
            8,
        );
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

use crate::{backend, error, Layout, Widget};

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
}

impl<S: AsRef<str>> Widget for Text<S> {
    fn render<B: backend::Backend>(
        &mut self,
        layout: Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        // Update just in case the layout is out of date
        if self.height(layout) == 1 {
            backend.write_all(self.wrapped.as_bytes())?;
        } else {
            for (i, line) in self.wrapped.lines().enumerate() {
                backend.set_cursor(layout.offset_x, layout.offset_y + i as u16)?;
                backend.write_all(line.as_bytes())?;
            }
        }

        Ok(())
    }

    fn height(&mut self, layout: crate::Layout) -> u16 {
        let width = layout.width - layout.offset_x;

        if self.width != width || self.line_offset != layout.line_offset {
            self.wrapped = fill(self.text.as_ref(), layout);
            self.width = width;
            self.line_offset = layout.line_offset;
        }

        self.wrapped.lines().count() as u16
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
    let s;

    let indent_len = layout.line_offset as usize;

    let indent = if SPACES.len() > indent_len {
        &SPACES[..indent_len]
    } else {
        s = " ".repeat(indent_len);
        &s[..]
    };

    let width = (layout.width - layout.offset_x) as usize;

    let mut text =
        textwrap::fill(text, textwrap::Options::new(width).initial_indent(indent));

    drop(text.drain(..indent.len()));

    text
}

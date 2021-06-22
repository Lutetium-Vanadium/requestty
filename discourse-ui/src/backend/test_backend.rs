use std::{
    fmt,
    io::{self, Write},
    ops,
};

use super::{ClearType, MoveDirection, Size};
use crate::{
    error,
    layout::Layout,
    style::{Attributes, Color},
    symbols,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    value: Option<char>,
    fg: Color,
    bg: Color,
    attributes: Attributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            value: None,
            fg: Color::Reset,
            bg: Color::Reset,
            attributes: Attributes::empty(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Cursor {
    x: u16,
    y: u16,
}

impl Cursor {
    fn to_linear(&self, width: u16) -> usize {
        (self.x + self.y * width) as usize
    }
}

impl From<Cursor> for (u16, u16) {
    fn from(c: Cursor) -> Self {
        (c.x, c.y)
    }
}

#[derive(Debug, Clone)]
pub struct TestBackend {
    cells: Vec<Cell>,
    cursor: Cursor,
    size: Size,
    raw: bool,
    hidden_cursor: bool,
    current_fg: Color,
    current_bg: Color,
    current_attributes: Attributes,
    viewport_start: usize,
}

impl PartialEq for TestBackend {
    /// Visual equality to another backend. This means that if the cells of both backends were
    /// rendered on a terminal, they would look the same. It however does not mean, that the hidden
    /// scrollback buffer is the same, or the current attributes are the same, or event the cursor
    /// position if it is hidden.
    fn eq(&self, other: &Self) -> bool {
        self.viewport() == other.viewport()
            && self.size == other.size
            && self.hidden_cursor == other.hidden_cursor
            && (self.hidden_cursor || self.cursor == other.cursor)
    }
}

impl Eq for TestBackend {}

impl TestBackend {
    pub fn new(size: Size) -> Self {
        Self::new_with_layout(size, Layout::new(0, size))
    }

    pub fn new_with_layout(size: Size, layout: Layout) -> Self {
        let mut this = Self {
            cells: [Cell::default()].repeat(size.area() as usize),
            cursor: Cursor::default(),
            size,
            raw: false,
            hidden_cursor: false,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            current_attributes: Attributes::empty(),
            viewport_start: 0,
        };

        this.move_x(layout.line_offset + layout.offset_x);
        this.move_y(layout.offset_y);

        this
    }

    /// Creates a new backend from the lines. There must be <=size.height lines, and <=size.width
    /// chars per line.
    ///
    /// note: It is not necessary to fill the lines so that it matches the dimensions of size
    /// exactly. Padding will be added to the end as required.
    pub fn from_lines(lines: &[&str], size: Size) -> Self {
        let mut backend = Self::new(size);

        assert!(lines.len() <= size.height as usize);
        let last_i = lines.len() - 1;

        for (i, line) in lines.iter().enumerate() {
            for c in line.chars() {
                assert!(backend.cursor.x + 1 < backend.size.width);
                backend.put_char(c);
            }
            if i < last_i {
                backend.move_x(0);
                backend.add_y(1);
            }
        }

        backend
    }

    pub fn reset_with_layout(&mut self, layout: Layout) {
        self.clear_range(..);
        self.move_x(layout.offset_x + layout.line_offset);
        self.move_y(layout.offset_y);
    }

    fn viewport(&self) -> &[Cell] {
        &self.cells[self.viewport_start..(self.viewport_start + self.size.area() as usize)]
    }

    fn move_x(&mut self, x: u16) {
        self.cursor.x = x.min(self.size.width - 1);
    }

    fn move_y(&mut self, y: u16) {
        self.cursor.y = y.min(self.size.height - 1);
    }

    fn add_x(&mut self, x: u16) {
        let x = self.cursor.x + x;
        let dy = x / self.size.width;
        self.cursor.x = x % self.size.width;
        self.move_y(self.cursor.y + dy);
    }

    fn sub_x(&mut self, x: u16) {
        self.cursor.x = self.cursor.x.saturating_sub(x);
    }

    fn add_y(&mut self, y: u16) {
        self.move_y(self.cursor.y + y)
    }

    fn sub_y(&mut self, y: u16) {
        self.cursor.y = self.cursor.y.saturating_sub(y);
    }

    fn cell_i(&self) -> usize {
        self.viewport_start + self.cursor.to_linear(self.size.width)
    }

    fn cell(&mut self) -> &mut Cell {
        let i = self.cell_i();
        &mut self.cells[i]
    }

    fn clear_range<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            ops::Bound::Included(&start) => start,
            ops::Bound::Excluded(start) => start.checked_add(1).unwrap(),
            ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            ops::Bound::Included(end) => end.checked_add(1).unwrap(),
            ops::Bound::Excluded(&end) => end,
            ops::Bound::Unbounded => self.cells.len(),
        };

        self.cells[start..end]
            .iter_mut()
            .for_each(|c| *c = Cell::default());
    }

    fn put_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.add_y(1);
                if !self.raw {
                    self.cursor.x = 0;
                }
            }
            '\r' => self.cursor.x = 0,
            '\t' => {
                let x = 8 + self.cursor.x - (self.cursor.x % 8);
                if x >= self.size.width && self.cursor.y < self.size.width - 1 {
                    self.cursor.x = 0;
                    self.cursor.y += 1;
                } else {
                    self.move_x(x);
                }
            }
            c => {
                self.cell().value = Some(c);
                self.cell().attributes = self.current_attributes;
                self.cell().fg = self.current_fg;
                self.cell().bg = self.current_bg;
                self.add_x(1);
            }
        }
    }

    pub fn assert_eq(&self, other: &Self) {
        if *self != *other {
            panic!(
                r#"assertion failed: `(left == right)`
 left:
{}
right:
{}
"#,
                self, other
            );
        }
    }
}

impl Write for TestBackend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        std::str::from_utf8(buf)
            .map_err(|_| io::ErrorKind::InvalidInput)?
            .chars()
            .for_each(|c| self.put_char(c));

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl super::Backend for TestBackend {
    fn enable_raw_mode(&mut self) -> error::Result<()> {
        self.raw = true;
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> error::Result<()> {
        self.raw = false;
        Ok(())
    }

    fn hide_cursor(&mut self) -> error::Result<()> {
        self.hidden_cursor = true;
        Ok(())
    }

    fn show_cursor(&mut self) -> error::Result<()> {
        self.hidden_cursor = false;
        Ok(())
    }

    fn get_cursor_pos(&mut self) -> error::Result<(u16, u16)> {
        Ok(self.cursor.into())
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> error::Result<()> {
        self.move_x(x);
        self.move_y(y);
        Ok(())
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
        match direction {
            MoveDirection::Up(n) => self.sub_y(n),
            MoveDirection::Down(n) => self.add_y(n),
            MoveDirection::Left(n) => self.sub_x(n),
            MoveDirection::Right(n) => self.add_y(n),
            MoveDirection::NextLine(n) => {
                self.cursor.x = 0;
                self.add_y(n);
            }
            MoveDirection::Column(n) => self.move_x(n),
            MoveDirection::PrevLine(n) => {
                self.cursor.x = 0;
                self.sub_y(n);
            }
        }
        Ok(())
    }

    fn scroll(&mut self, dist: i16) -> error::Result<()> {
        if dist.is_positive() {
            self.viewport_start += (dist as usize) * self.size.width as usize;
            let new_len = self.viewport_start + self.size.area() as usize;

            if new_len > self.cells.len() {
                self.cells.resize_with(new_len, Cell::default)
            };
        } else {
            self.viewport_start = self
                .viewport_start
                .saturating_sub((-dist) as usize * self.size.width as usize);
        }
        Ok(())
    }

    fn set_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        self.current_attributes = attributes;
        Ok(())
    }

    fn set_fg(&mut self, color: Color) -> error::Result<()> {
        self.current_fg = color;
        Ok(())
    }

    fn set_bg(&mut self, color: Color) -> error::Result<()> {
        self.current_bg = color;
        Ok(())
    }

    fn clear(&mut self, clear_type: ClearType) -> error::Result<()> {
        match clear_type {
            ClearType::All => self.clear_range(..),
            ClearType::FromCursorDown => self.clear_range(self.cell_i()..),
            ClearType::FromCursorUp => self.clear_range(..=self.cell_i()),
            ClearType::CurrentLine => {
                let s = (self.cursor.y * self.size.width) as usize;
                let e = ((self.cursor.y + 1) * self.size.width) as usize;
                self.clear_range(s..e)
            }
            ClearType::UntilNewLine => {
                let e = ((self.cursor.y + 1) * self.size.width) as usize;
                self.clear_range(self.cell_i()..e)
            }
        }
        Ok(())
    }

    fn size(&self) -> error::Result<Size> {
        Ok(self.size)
    }
}

impl fmt::Display for TestBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = Vec::with_capacity(self.size.area() as usize);

        if let Err(e) = self.write_to_buf(&mut buf) {
            return write!(f, "<could not render TestBackend: {}>", e);
        }

        match std::str::from_utf8(&buf) {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<could not render TestBackend: {}>", e),
        }
    }
}

impl TestBackend {
    pub fn write_to_buf<W: Write>(&self, mut buf: W) -> error::Result<()> {
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut attributes = Attributes::empty();

        let cursor = if self.hidden_cursor {
            usize::MAX
        } else {
            self.cursor.to_linear(self.size.width) as usize
        };

        let width = self.size.width as usize;

        write!(buf, "{}", symbols::BOX_LIGHT_TOP_LEFT)?;
        for _ in 0..self.size.width {
            write!(buf, "{}", symbols::BOX_LIGHT_HORIZONTAL)?;
        }
        writeln!(buf, "{}", symbols::BOX_LIGHT_TOP_RIGHT)?;

        for (i, cell) in self.viewport().iter().enumerate() {
            if i % width == 0 {
                write!(buf, "{}", symbols::BOX_LIGHT_VERTICAL)?;
            }

            if cell.attributes != attributes {
                display_ops::set_attributes(attributes, cell.attributes, &mut buf)?;
                attributes = cell.attributes;
            }

            let (cell_fg, cell_bg) = if i == cursor {
                (
                    cell.bg,
                    match cell.fg {
                        Color::Reset => Color::Grey,
                        c => c,
                    },
                )
            } else {
                (cell.fg, cell.bg)
            };

            if cell_fg != fg {
                display_ops::write_fg(cell_fg, &mut buf)?;
                fg = cell_fg;
            }
            if cell_bg != bg {
                display_ops::write_bg(cell_bg, &mut buf)?;
                bg = cell_bg;
            }

            write!(buf, "{}", cell.value.unwrap_or(' '))?;

            if (i + 1) % width == 0 {
                if !attributes.is_empty() {
                    display_ops::set_attributes(attributes, Attributes::empty(), &mut buf)?;
                    attributes = Attributes::empty();
                }
                if fg != Color::Reset {
                    fg = Color::Reset;
                    display_ops::write_fg(fg, &mut buf)?;
                }
                if bg != Color::Reset {
                    bg = Color::Reset;
                    display_ops::write_bg(bg, &mut buf)?;
                }
                writeln!(buf, "{}", symbols::BOX_LIGHT_VERTICAL)?;
            }
        }

        write!(buf, "{}", symbols::BOX_LIGHT_BOTTOM_LEFT)?;
        for _ in 0..self.size.width {
            write!(buf, "{}", symbols::BOX_LIGHT_HORIZONTAL)?;
        }
        write!(buf, "{}", symbols::BOX_LIGHT_BOTTOM_RIGHT)?;

        buf.flush().map_err(Into::into)
    }
}

#[cfg(feature = "crossterm")]
mod display_ops {
    use std::io::Write;

    use crossterm::{
        queue,
        style::{SetBackgroundColor, SetForegroundColor},
    };

    use crate::{error, style::Color};

    pub(super) use crate::backend::crossterm::set_attributes;

    pub(super) fn write_fg<W: Write>(fg: Color, mut w: W) -> error::Result<()> {
        queue!(w, SetForegroundColor(fg.into())).map_err(Into::into)
    }

    pub(super) fn write_bg<W: Write>(bg: Color, mut w: W) -> error::Result<()> {
        queue!(w, SetBackgroundColor(bg.into())).map_err(Into::into)
    }
}

#[cfg(feature = "termion")]
mod display_ops {
    use std::io::Write;

    use crate::{backend::termion, error, style::Color};

    pub(super) use self::termion::set_attributes;

    pub(super) fn write_fg<W: Write>(fg: Color, mut w: W) -> error::Result<()> {
        write!(w, "{}", termion::Fg(fg)).map_err(Into::into)
    }

    pub(super) fn write_bg<W: Write>(bg: Color, mut w: W) -> error::Result<()> {
        write!(w, "{}", termion::Bg(bg)).map_err(Into::into)
    }
}

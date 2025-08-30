use std::{
    io::{self, Write},
    ops,
};

use super::{Backend, ClearType, DisplayBackend, MoveDirection, Size};
use crate::{
    layout::Layout,
    style::{Attributes, Color},
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
    fn to_linear(self, width: u16) -> usize {
        (self.x + self.y * width) as usize
    }
}

impl From<Cursor> for (u16, u16) {
    fn from(c: Cursor) -> Self {
        (c.x, c.y)
    }
}

/// A backend that can be used for tests.
///
/// When asserting equality, it is recommended to use [`TestBackend::assert_eq`] or
/// [`assert_backend_snapshot`] instead of [`assert_eq`].
///
/// [`assert_backend_snapshot`]: crate::assert_backend_snapshot
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
    /// Creates a new `TestBackend`
    pub fn new(size: Size) -> Self {
        Self::new_with_layout(size, Layout::new(0, size))
    }

    /// Creates a new `TestBackend` with the cursor starting at the offsets given by the layout.
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

    /// Creates a new `TestBackend` from the lines. There must be `<= size.height` lines, and
    /// `<= size.width` chars per line.
    ///
    /// It is not necessary to fill the lines so that it matches the dimensions of size exactly.
    /// Padding will be added to the end as required.
    ///
    /// # Panics
    ///
    /// It panics if there are more than `size.height` lines or more than `size.width` chars per
    /// line.
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

    /// Clears all the cells and moves the cursor to the offsets given by the layout.
    pub fn reset_with_layout(&mut self, layout: Layout) {
        self.clear_range(..);
        self.move_x(layout.offset_x + layout.line_offset);
        self.move_y(layout.offset_y);
    }

    fn viewport(&self) -> &[Cell] {
        &self.cells[self.viewport_start..(self.viewport_start + self.size.area() as usize)]
    }

    fn move_x(&mut self, x: u16) {
        // wrapping_sub to allow testing 0 sized terminals
        self.cursor.x = x.min(self.size.width.wrapping_sub(1));
    }

    fn move_y(&mut self, y: u16) {
        // wrapping_sub to allow testing 0 sized terminals
        self.cursor.y = y.min(self.size.height.wrapping_sub(1));
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

    #[cfg(any(feature = "crossterm", feature = "termion"))]
    fn assertion_failed(&self, other: &Self) {
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

    #[cfg(not(any(feature = "crossterm", feature = "termion")))]
    fn assertion_failed(&self, other: &Self) {
        panic!(
            r#"assertion failed: `(left == right)`
 left:
`TestBackend` {:p}
right:
`TestBackend` {:p}

Enable any of the default backends to view what the `TestBackend`s looked like
"#,
            self, other
        );
    }

    /// Asserts that two `TestBackend`s are equal to each other, otherwise it panics printing what
    /// the backend would look like.
    pub fn assert_eq(&self, other: &Self) {
        if *self != *other {
            self.assertion_failed(other);
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

impl DisplayBackend for TestBackend {
    fn set_attributes(&mut self, attributes: Attributes) -> io::Result<()> {
        self.current_attributes = attributes;
        Ok(())
    }

    fn set_fg(&mut self, color: Color) -> io::Result<()> {
        self.current_fg = color;
        Ok(())
    }

    fn set_bg(&mut self, color: Color) -> io::Result<()> {
        self.current_bg = color;
        Ok(())
    }
}

impl Backend for TestBackend {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        self.raw = true;
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        self.raw = false;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.hidden_cursor = true;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.hidden_cursor = false;
        Ok(())
    }

    fn get_cursor_pos(&mut self) -> io::Result<(u16, u16)> {
        Ok(self.cursor.into())
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.move_x(x);
        self.move_y(y);
        Ok(())
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> io::Result<()> {
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

    fn scroll(&mut self, dist: i16) -> io::Result<()> {
        if dist.is_positive() {
            self.viewport_start = self
                .viewport_start
                .saturating_sub(dist as usize * self.size.width as usize);
        } else {
            self.viewport_start += (-dist as usize) * self.size.width as usize;
            let new_len = self.viewport_start + self.size.area() as usize;

            if new_len > self.cells.len() {
                self.cells.resize_with(new_len, Cell::default)
            };
        }
        Ok(())
    }

    fn clear(&mut self, clear_type: ClearType) -> io::Result<()> {
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

    fn size(&self) -> io::Result<Size> {
        Ok(self.size)
    }
}

#[cfg(any(feature = "crossterm", feature = "termion"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
impl std::fmt::Display for TestBackend {
    /// Writes all the cells of the `TestBackend` using [`write_to_buf`].
    ///
    /// A screenshot of what the printed output looks like:
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/test-backend-rendered.png)
    ///
    /// [`write_to_buf`]: TestBackend::write_to_buf
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

fn map_reset(c: Color, to: Color) -> Color {
    match c {
        Color::Reset => to,
        c => c,
    }
}

impl TestBackend {
    /// Writes all the cells of the `TestBackend` to the given backend.
    ///
    /// A screenshot of what the printed output looks like:
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/test-backend-rendered.png)
    pub fn write_to_backend<B: DisplayBackend>(&self, mut backend: B) -> io::Result<()> {
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut attributes = Attributes::empty();

        let cursor = if self.hidden_cursor {
            usize::MAX
        } else {
            self.cursor.to_linear(self.size.width)
        };

        let width = self.size.width as usize;

        let symbol_set = crate::symbols::current();

        write!(backend, "{}", symbol_set.box_top_left)?;
        for _ in 0..self.size.width {
            write!(backend, "{}", symbol_set.box_horizontal)?;
        }
        writeln!(backend, "{}", symbol_set.box_top_right)?;

        for (i, cell) in self.viewport().iter().enumerate() {
            if i % width == 0 {
                write!(backend, "{}", symbol_set.box_vertical)?;
            }

            if cell.attributes != attributes {
                backend.set_attributes(cell.attributes)?;
                attributes = cell.attributes;
            }

            let (cell_fg, cell_bg) = if i == cursor {
                (
                    map_reset(cell.bg, Color::Black),
                    map_reset(cell.fg, Color::Grey),
                )
            } else {
                (cell.fg, cell.bg)
            };

            if cell_fg != fg {
                backend.set_fg(cell_fg)?;
                fg = cell_fg;
            }
            if cell_bg != bg {
                backend.set_bg(cell_bg)?;
                bg = cell_bg;
            }

            write!(backend, "{}", cell.value.unwrap_or(' '))?;

            if (i + 1) % width == 0 {
                if !attributes.is_empty() {
                    backend.set_attributes(Attributes::empty())?;
                    attributes = Attributes::empty();
                }
                if fg != Color::Reset {
                    fg = Color::Reset;
                    backend.set_fg(fg)?;
                }
                if bg != Color::Reset {
                    bg = Color::Reset;
                    backend.set_bg(bg)?;
                }
                writeln!(backend, "{}", symbol_set.box_vertical)?;
            }
        }

        write!(backend, "{}", symbol_set.box_bottom_left)?;
        for _ in 0..self.size.width {
            write!(backend, "{}", symbol_set.box_horizontal)?;
        }
        write!(backend, "{}", symbol_set.box_bottom_right)?;

        backend.flush()
    }

    /// Writes all the cells of the `TestBackend` with the default backend (see [`get_backend`]).
    ///
    /// A screenshot of what the printed output looks like:
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/test-backend-rendered.png)
    ///
    /// [`get_backend`]: crate::backend::get_backend
    #[cfg(any(feature = "crossterm", feature = "termion"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
    pub fn write_to_buf<W: Write>(&self, buf: W) -> io::Result<()> {
        #[cfg(feature = "crossterm")]
        return self.write_to_backend(super::CrosstermBackend::new(buf));
        #[cfg(all(not(feature = "crossterm"), feature = "termion"))]
        return self.write_to_backend(super::TermionDisplayBackend::new(buf));
    }
}

//! A module to represent a terminal and operations on it.

use std::{fmt::Display, io};

/// Gets the default [`Backend`] based on the features enabled.
pub fn get_backend<W: io::Write>(buf: W) -> io::Result<impl Backend> {
    #[cfg(feature = "crossterm")]
    return Ok(CrosstermBackend::new(buf));

    // XXX: Only works when crossterm and termion are the only two available backends
    //
    // Instead of directly checking for termion, we check for not crossterm so that compiling
    // (documentation) with both features enabled will not error
    #[cfg(not(feature = "crossterm"))]
    return TermionBackend::new(buf);
}

mod test_backend;
pub use test_backend::TestBackend;

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "termion")]
#[cfg_attr(docsrs, doc(cfg(feature = "termion")))]
pub use self::termion::TermionBackend;

#[cfg(feature = "crossterm")]
mod crossterm;

#[cfg(feature = "crossterm")]
#[cfg_attr(docsrs, doc(cfg(feature = "crossterm")))]
pub use self::crossterm::CrosstermBackend;

use crate::style::{Attributes, Color, Styled};

/// A 2D size.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
#[allow(missing_docs)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    /// The area of the size
    pub fn area(self) -> u16 {
        self.width * self.height
    }
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Size { width, height }
    }
}

/// The different parts of the terminal that can be cleared at once.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ClearType {
    /// All cells.
    All,
    /// All cells from the cursor position downwards.
    FromCursorDown,
    /// All cells from the cursor position upwards.
    FromCursorUp,
    /// All cells at the cursor row.
    CurrentLine,
    /// All cells from the cursor position until the new line.
    UntilNewLine,
}

/// The directions the terminal cursor can be moved relative to the current position.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MoveDirection {
    /// Moves a given number of rows up.
    Up(u16),
    /// Moves a given number of rows down.
    Down(u16),
    /// Moves a given number of columns left.
    Left(u16),
    /// Moves a given number of columns right.
    Right(u16),
    /// Moves a given number of rows down and goes to the start of the line.
    NextLine(u16),
    /// Moves a given number of rows up and goes to the start of the line.
    PrevLine(u16),
    /// Goes to a given column.
    Column(u16),
}

/// A trait to represent a terminal that can be rendered to.
pub trait Backend: io::Write {
    /// Enables raw mode.
    fn enable_raw_mode(&mut self) -> io::Result<()>;
    /// Disables raw mode.
    fn disable_raw_mode(&mut self) -> io::Result<()>;
    /// Hides the cursor.
    fn hide_cursor(&mut self) -> io::Result<()>;
    /// Shows the cursor.
    fn show_cursor(&mut self) -> io::Result<()>;

    /// Gets the cursor position as (col, row). The top-left cell is (0, 0).
    fn get_cursor_pos(&mut self) -> io::Result<(u16, u16)>;
    /// Moves the cursor to given position. The top-left cell is (0, 0).
    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()>;
    /// Moves the cursor relative to the current position as per the `direction`.
    fn move_cursor(&mut self, direction: MoveDirection) -> io::Result<()> {
        default_move_cursor(self, direction)
    }
    /// Scrolls the terminal the given number of rows.
    ///
    /// A negative number means the terminal scrolls upwards, while a positive number means the
    /// terminal scrolls downwards.
    fn scroll(&mut self, dist: i16) -> io::Result<()>;

    /// Sets the given `attributes` removing ones which were previous applied.
    fn set_attributes(&mut self, attributes: Attributes) -> io::Result<()>;
    /// Sets the foreground color.
    fn set_fg(&mut self, color: Color) -> io::Result<()>;
    /// Sets the background color.
    fn set_bg(&mut self, color: Color) -> io::Result<()>;
    /// Write a styled object to the backend.
    ///
    /// See also [`Styled`] and [`Stylize`].
    ///
    /// [`Stylize`]: crate::style::Stylize
    fn write_styled(&mut self, styled: &Styled<dyn Display + '_>) -> io::Result<()> {
        styled.write(self)
    }

    /// Clears the cells given by clear_type
    fn clear(&mut self, clear_type: ClearType) -> io::Result<()>;
    /// Gets the size of the terminal in rows and columns.
    fn size(&self) -> io::Result<Size>;
}

fn default_move_cursor<B: Backend + ?Sized>(
    backend: &mut B,
    direction: MoveDirection,
) -> io::Result<()> {
    let (mut x, mut y) = backend.get_cursor_pos()?;

    match direction {
        MoveDirection::Up(dy) => y = y.saturating_sub(dy),
        MoveDirection::Down(dy) => y = y.saturating_add(dy),
        MoveDirection::Left(dx) => x = x.saturating_sub(dx),
        MoveDirection::Right(dx) => x = x.saturating_add(dx),
        MoveDirection::NextLine(dy) => {
            x = 0;
            y = y.saturating_add(dy);
        }
        MoveDirection::Column(new_x) => x = new_x,
        MoveDirection::PrevLine(dy) => {
            x = 0;
            y = y.saturating_sub(dy);
        }
    }

    backend.move_cursor_to(x, y)
}

impl<'a, B: Backend> Backend for &'a mut B {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        (**self).enable_raw_mode()
    }
    fn disable_raw_mode(&mut self) -> io::Result<()> {
        (**self).disable_raw_mode()
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        (**self).hide_cursor()
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        (**self).show_cursor()
    }
    fn get_cursor_pos(&mut self) -> io::Result<(u16, u16)> {
        (**self).get_cursor_pos()
    }
    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        (**self).move_cursor_to(x, y)
    }
    fn move_cursor(&mut self, direction: MoveDirection) -> io::Result<()> {
        (**self).move_cursor(direction)
    }
    fn scroll(&mut self, dist: i16) -> io::Result<()> {
        (**self).scroll(dist)
    }
    fn set_attributes(&mut self, attributes: Attributes) -> io::Result<()> {
        (**self).set_attributes(attributes)
    }
    fn set_fg(&mut self, color: Color) -> io::Result<()> {
        (**self).set_fg(color)
    }
    fn set_bg(&mut self, color: Color) -> io::Result<()> {
        (**self).set_bg(color)
    }
    fn write_styled(&mut self, styled: &Styled<dyn Display + '_>) -> io::Result<()> {
        (**self).write_styled(styled)
    }
    fn clear(&mut self, clear_type: ClearType) -> io::Result<()> {
        (**self).clear(clear_type)
    }
    fn size(&self) -> io::Result<Size> {
        (**self).size()
    }
}

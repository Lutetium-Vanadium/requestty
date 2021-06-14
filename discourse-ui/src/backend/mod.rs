use std::fmt::Display;

use crate::error;

pub fn get_backend<W: std::io::Write>(buf: W) -> error::Result<impl Backend> {
    #[cfg(feature = "crossterm")]
    type Backend<W> = CrosstermBackend<W>;
    #[cfg(feature = "termion")]
    type Backend<W> = TermionBackend<W>;

    Backend::new(buf)
}

mod test_backend;
pub use test_backend::{TestBackend, TestBackendOp};

#[cfg(feature = "termion")]
mod termion;
#[cfg(feature = "termion")]
pub use self::termion::TermionBackend;

#[cfg(feature = "crossterm")]
mod crossterm;
#[cfg(feature = "crossterm")]
pub use self::crossterm::CrosstermBackend;

use crate::style::{Attributes, Color, Styled};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Size { width, height }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ClearType {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MoveDirection {
    Up(u16),
    Down(u16),
    Left(u16),
    Right(u16),
    NextLine(u16),
    Column(u16),
    PrevLine(u16),
}

pub trait Backend: std::io::Write {
    fn enable_raw_mode(&mut self) -> error::Result<()>;
    fn disable_raw_mode(&mut self) -> error::Result<()>;
    fn hide_cursor(&mut self) -> error::Result<()>;
    fn show_cursor(&mut self) -> error::Result<()>;

    fn get_cursor_pos(&mut self) -> error::Result<(u16, u16)>;
    fn move_cursor_to(&mut self, x: u16, y: u16) -> error::Result<()>;
    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
        default_move_cursor(self, direction)
    }
    fn scroll(&mut self, dist: i16) -> error::Result<()>;
    fn set_attributes(&mut self, attributes: Attributes) -> error::Result<()>;
    fn remove_attributes(&mut self, attributes: Attributes) -> error::Result<()>;
    fn set_fg(&mut self, color: Color) -> error::Result<()>;
    fn set_bg(&mut self, color: Color) -> error::Result<()>;
    fn write_styled(
        &mut self,
        styled: &Styled<dyn Display + '_>,
    ) -> error::Result<()> {
        styled.write(self)
    }
    fn clear(&mut self, clear_type: ClearType) -> error::Result<()>;
    fn size(&self) -> error::Result<Size>;
}

fn default_move_cursor<B: Backend + ?Sized>(
    backend: &mut B,
    direction: MoveDirection,
) -> error::Result<()> {
    // There are a lot of `MoveDirection::NextLine(1)`, so this will slightly speed up
    // the rendering process as the cursor doesn't need to be gotten every time
    if let MoveDirection::NextLine(1) = direction {
        return write!(backend, "\n\r").map_err(Into::into);
    }

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
    fn enable_raw_mode(&mut self) -> error::Result<()> {
        (**self).enable_raw_mode()
    }
    fn disable_raw_mode(&mut self) -> error::Result<()> {
        (**self).disable_raw_mode()
    }
    fn hide_cursor(&mut self) -> error::Result<()> {
        (**self).hide_cursor()
    }
    fn show_cursor(&mut self) -> error::Result<()> {
        (**self).show_cursor()
    }
    fn get_cursor_pos(&mut self) -> error::Result<(u16, u16)> {
        (**self).get_cursor_pos()
    }
    fn move_cursor_to(&mut self, x: u16, y: u16) -> error::Result<()> {
        (**self).move_cursor_to(x, y)
    }
    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
        (**self).move_cursor(direction)
    }
    fn scroll(&mut self, dist: i16) -> error::Result<()> {
        (**self).scroll(dist)
    }
    fn set_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        (**self).set_attributes(attributes)
    }
    fn remove_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        (**self).remove_attributes(attributes)
    }
    fn set_fg(&mut self, color: Color) -> error::Result<()> {
        (**self).set_fg(color)
    }
    fn set_bg(&mut self, color: Color) -> error::Result<()> {
        (**self).set_bg(color)
    }
    fn write_styled(
        &mut self,
        styled: &Styled<dyn Display + '_>,
    ) -> error::Result<()> {
        (**self).write_styled(styled)
    }
    fn clear(&mut self, clear_type: ClearType) -> error::Result<()> {
        (**self).clear(clear_type)
    }
    fn size(&self) -> error::Result<Size> {
        (**self).size()
    }
}

use std::{
    cmp::Ordering,
    io::{self, Write},
};

use crossterm::{
    cursor, queue,
    style::{
        Attribute as CAttribute, Color as CColor, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal,
};

use super::{Attributes, Backend, ClearType, Color, DisplayBackend, MoveDirection, Size};

/// A backend that uses the `crossterm` library.
#[derive(Debug, Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "crossterm")))]
pub struct CrosstermBackend<W> {
    buffer: W,
    attributes: Attributes,
}

impl<W> CrosstermBackend<W> {
    /// Creates a new [`CrosstermBackend`]
    pub fn new(buffer: W) -> CrosstermBackend<W> {
        CrosstermBackend {
            buffer,
            attributes: Attributes::empty(),
        }
    }
}

impl<W: Write> Write for CrosstermBackend<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl<W: Write> DisplayBackend for CrosstermBackend<W> {
    fn set_attributes(&mut self, attributes: Attributes) -> io::Result<()> {
        set_attributes(self.attributes, attributes, &mut self.buffer)?;
        self.attributes = attributes;
        Ok(())
    }

    fn set_fg(&mut self, color: Color) -> io::Result<()> {
        queue!(self.buffer, SetForegroundColor(color.into()))
    }

    fn set_bg(&mut self, color: Color) -> io::Result<()> {
        queue!(self.buffer, SetBackgroundColor(color.into()))
    }
}

impl<W: Write> Backend for CrosstermBackend<W> {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        terminal::disable_raw_mode()
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        queue!(self.buffer, cursor::Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        queue!(self.buffer, cursor::Show)
    }

    fn get_cursor_pos(&mut self) -> io::Result<(u16, u16)> {
        cursor::position()
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        queue!(self.buffer, cursor::MoveTo(x, y))
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> io::Result<()> {
        match direction {
            MoveDirection::Up(n) => queue!(self.buffer, cursor::MoveUp(n)),
            MoveDirection::Down(n) => queue!(self.buffer, cursor::MoveDown(n)),
            MoveDirection::Left(n) => queue!(self.buffer, cursor::MoveLeft(n)),
            MoveDirection::Right(n) => queue!(self.buffer, cursor::MoveRight(n)),
            MoveDirection::NextLine(n) => {
                queue!(self.buffer, cursor::MoveToNextLine(n))
            }
            MoveDirection::Column(n) => queue!(self.buffer, cursor::MoveToColumn(n)),
            MoveDirection::PrevLine(n) => {
                queue!(self.buffer, cursor::MoveToPreviousLine(n))
            }
        }
    }

    fn scroll(&mut self, dist: i16) -> io::Result<()> {
        match dist.cmp(&0) {
            Ordering::Greater => {
                queue!(self.buffer, terminal::ScrollDown(dist as u16))
            }
            Ordering::Less => {
                queue!(self.buffer, terminal::ScrollUp(-dist as u16))
            }
            Ordering::Equal => Ok(()),
        }
    }

    fn clear(&mut self, clear_type: ClearType) -> io::Result<()> {
        queue!(self.buffer, terminal::Clear(clear_type.into()))
    }

    fn size(&self) -> io::Result<Size> {
        terminal::size().map(Into::into)
    }
}

impl From<Color> for CColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => CColor::Reset,
            Color::Black => CColor::Black,
            Color::Red => CColor::DarkRed,
            Color::Green => CColor::DarkGreen,
            Color::Yellow => CColor::DarkYellow,
            Color::Blue => CColor::DarkBlue,
            Color::Magenta => CColor::DarkMagenta,
            Color::Cyan => CColor::DarkCyan,
            Color::Grey => CColor::Grey,
            Color::DarkGrey => CColor::DarkGrey,
            Color::LightRed => CColor::Red,
            Color::LightGreen => CColor::Green,
            Color::LightBlue => CColor::Blue,
            Color::LightYellow => CColor::Yellow,
            Color::LightMagenta => CColor::Magenta,
            Color::LightCyan => CColor::Cyan,
            Color::White => CColor::White,
            Color::Ansi(i) => CColor::AnsiValue(i),
            Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
        }
    }
}

impl From<ClearType> for terminal::ClearType {
    fn from(clear: ClearType) -> Self {
        match clear {
            ClearType::All => terminal::ClearType::All,
            ClearType::FromCursorDown => terminal::ClearType::FromCursorDown,
            ClearType::FromCursorUp => terminal::ClearType::FromCursorUp,
            ClearType::CurrentLine => terminal::ClearType::CurrentLine,
            ClearType::UntilNewLine => terminal::ClearType::UntilNewLine,
        }
    }
}

pub(super) fn set_attributes<W: Write>(
    from: Attributes,
    to: Attributes,
    mut w: W,
) -> io::Result<()> {
    let diff = from.diff(to);
    if diff.to_remove.contains(Attributes::REVERSED) {
        queue!(w, SetAttribute(CAttribute::NoReverse))?;
    }
    if diff.to_remove.contains(Attributes::BOLD) {
        queue!(w, SetAttribute(CAttribute::NormalIntensity))?;
        if to.contains(Attributes::DIM) {
            queue!(w, SetAttribute(CAttribute::Dim))?;
        }
    }
    if diff.to_remove.contains(Attributes::ITALIC) {
        queue!(w, SetAttribute(CAttribute::NoItalic))?;
    }
    if diff.to_remove.contains(Attributes::UNDERLINED) {
        queue!(w, SetAttribute(CAttribute::NoUnderline))?;
    }
    if diff.to_remove.contains(Attributes::DIM) {
        queue!(w, SetAttribute(CAttribute::NormalIntensity))?;
    }
    if diff.to_remove.contains(Attributes::CROSSED_OUT) {
        queue!(w, SetAttribute(CAttribute::NotCrossedOut))?;
    }
    if diff.to_remove.contains(Attributes::SLOW_BLINK)
        || diff.to_remove.contains(Attributes::RAPID_BLINK)
    {
        queue!(w, SetAttribute(CAttribute::NoBlink))?;
    }

    if diff.to_add.contains(Attributes::REVERSED) {
        queue!(w, SetAttribute(CAttribute::Reverse))?;
    }
    if diff.to_add.contains(Attributes::BOLD) {
        queue!(w, SetAttribute(CAttribute::Bold))?;
    }
    if diff.to_add.contains(Attributes::ITALIC) {
        queue!(w, SetAttribute(CAttribute::Italic))?;
    }
    if diff.to_add.contains(Attributes::UNDERLINED) {
        queue!(w, SetAttribute(CAttribute::Underlined))?;
    }
    if diff.to_add.contains(Attributes::DIM) {
        queue!(w, SetAttribute(CAttribute::Dim))?;
    }
    if diff.to_add.contains(Attributes::CROSSED_OUT) {
        queue!(w, SetAttribute(CAttribute::CrossedOut))?;
    }
    if diff.to_add.contains(Attributes::SLOW_BLINK) {
        queue!(w, SetAttribute(CAttribute::SlowBlink))?;
    }
    if diff.to_add.contains(Attributes::RAPID_BLINK) {
        queue!(w, SetAttribute(CAttribute::RapidBlink))?;
    }

    Ok(())
}

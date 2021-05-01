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

use super::{Attributes, Backend, ClearType, Color, MoveDirection, Size};
use crate::error;

pub struct CrosstermBackend<W> {
    buffer: W,
}

impl<W> CrosstermBackend<W> {
    pub fn new(buffer: W) -> error::Result<CrosstermBackend<W>> {
        Ok(CrosstermBackend { buffer })
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

impl<W: Write> Backend for CrosstermBackend<W> {
    fn enable_raw_mode(&mut self) -> error::Result<()> {
        terminal::enable_raw_mode().map_err(Into::into)
    }

    fn disable_raw_mode(&mut self) -> error::Result<()> {
        terminal::disable_raw_mode().map_err(Into::into)
    }

    fn hide_cursor(&mut self) -> error::Result<()> {
        queue!(self.buffer, cursor::Hide).map_err(Into::into)
    }

    fn show_cursor(&mut self) -> error::Result<()> {
        queue!(self.buffer, cursor::Show).map_err(Into::into)
    }

    fn get_cursor(&mut self) -> error::Result<(u16, u16)> {
        cursor::position().map_err(Into::into)
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> error::Result<()> {
        queue!(self.buffer, cursor::MoveTo(x, y)).map_err(Into::into)
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
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
        .map_err(Into::into)
    }

    fn scroll(&mut self, dist: i32) -> error::Result<()> {
        match dist.cmp(&0) {
            Ordering::Greater => {
                queue!(self.buffer, terminal::ScrollDown(dist as u16))
            }
            Ordering::Less => {
                queue!(self.buffer, terminal::ScrollUp(-dist as u16))
            }
            Ordering::Equal => Ok(()),
        }
        .map_err(Into::into)
    }

    fn set_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        set_attributes(attributes, &mut self.buffer)
    }

    fn remove_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        remove_attributes(attributes, &mut self.buffer)
    }

    fn set_fg(&mut self, color: Color) -> error::Result<()> {
        queue!(self.buffer, SetForegroundColor(color.into())).map_err(Into::into)
    }

    fn set_bg(&mut self, color: Color) -> error::Result<()> {
        queue!(self.buffer, SetBackgroundColor(color.into())).map_err(Into::into)
    }

    fn clear(&mut self, clear_type: ClearType) -> error::Result<()> {
        queue!(self.buffer, terminal::Clear(clear_type.into())).map_err(Into::into)
    }

    fn size(&self) -> error::Result<Size> {
        terminal::size().map(Into::into).map_err(Into::into)
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

fn set_attributes<W: Write>(attributes: Attributes, mut w: W) -> error::Result<()> {
    if attributes.contains(Attributes::RESET) {
        return queue!(w, SetAttribute(CAttribute::Reset)).map_err(Into::into);
    }

    if attributes.contains(Attributes::REVERSED) {
        queue!(w, SetAttribute(CAttribute::Reverse))?;
    }
    if attributes.contains(Attributes::BOLD) {
        queue!(w, SetAttribute(CAttribute::Bold))?;
    }
    if attributes.contains(Attributes::ITALIC) {
        queue!(w, SetAttribute(CAttribute::Italic))?;
    }
    if attributes.contains(Attributes::UNDERLINED) {
        queue!(w, SetAttribute(CAttribute::Underlined))?;
    }
    if attributes.contains(Attributes::DIM) {
        queue!(w, SetAttribute(CAttribute::Dim))?;
    }
    if attributes.contains(Attributes::CROSSED_OUT) {
        queue!(w, SetAttribute(CAttribute::CrossedOut))?;
    }
    if attributes.contains(Attributes::SLOW_BLINK) {
        queue!(w, SetAttribute(CAttribute::SlowBlink))?;
    }
    if attributes.contains(Attributes::RAPID_BLINK) {
        queue!(w, SetAttribute(CAttribute::RapidBlink))?;
    }
    if attributes.contains(Attributes::HIDDEN) {
        queue!(w, SetAttribute(CAttribute::Hidden))?;
    }

    Ok(())
}

fn remove_attributes<W: Write>(
    attributes: Attributes,
    mut w: W,
) -> error::Result<()> {
    if attributes.contains(Attributes::RESET) {
        return queue!(w, SetAttribute(CAttribute::Reset)).map_err(Into::into);
    }

    if attributes.contains(Attributes::REVERSED) {
        queue!(w, SetAttribute(CAttribute::NoReverse))?;
    }
    if attributes.contains(Attributes::BOLD) {
        queue!(w, SetAttribute(CAttribute::NoBold))?;
    }
    if attributes.contains(Attributes::ITALIC) {
        queue!(w, SetAttribute(CAttribute::NoItalic))?;
    }
    if attributes.contains(Attributes::UNDERLINED) {
        queue!(w, SetAttribute(CAttribute::NoUnderline))?;
    }
    if attributes.contains(Attributes::DIM) {
        queue!(w, SetAttribute(CAttribute::NormalIntensity))?;
    }
    if attributes.contains(Attributes::CROSSED_OUT) {
        queue!(w, SetAttribute(CAttribute::NotCrossedOut))?;
    }
    if attributes.contains(Attributes::SLOW_BLINK | Attributes::RAPID_BLINK) {
        queue!(w, SetAttribute(CAttribute::NoBlink))?;
    }
    if attributes.contains(Attributes::HIDDEN) {
        queue!(w, SetAttribute(CAttribute::NoHidden))?;
    }

    Ok(())
}

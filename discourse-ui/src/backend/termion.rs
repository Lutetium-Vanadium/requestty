use std::{
    cmp::Ordering,
    fmt,
    io::{self, Write},
};

use termion::{
    clear, color, cursor,
    raw::{IntoRawMode, RawTerminal},
    scroll, style,
};

use super::{Attributes, Backend, ClearType, Color, MoveDirection, Size};
use crate::error;

pub struct TermionBackend<W: Write> {
    buffer: RawTerminal<W>,
}

impl<W: Write> TermionBackend<W> {
    pub fn new(buffer: W) -> error::Result<TermionBackend<W>> {
        let buffer = buffer.into_raw_mode()?;
        buffer.suspend_raw_mode()?;
        Ok(TermionBackend { buffer })
    }
}

impl<W: Write> Write for TermionBackend<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl<W: Write> Backend for TermionBackend<W> {
    fn enable_raw_mode(&mut self) -> error::Result<()> {
        self.buffer.activate_raw_mode().map_err(Into::into)
    }

    fn disable_raw_mode(&mut self) -> error::Result<()> {
        self.buffer.suspend_raw_mode().map_err(Into::into)
    }

    fn hide_cursor(&mut self) -> error::Result<()> {
        write!(self.buffer, "{}", cursor::Hide).map_err(Into::into)
    }

    fn show_cursor(&mut self) -> error::Result<()> {
        write!(self.buffer, "{}", cursor::Show).map_err(Into::into)
    }

    fn get_cursor_pos(&mut self) -> error::Result<(u16, u16)> {
        cursor::DetectCursorPos::cursor_pos(&mut self.buffer)
            // 0 index the position
            .map(|(x, y)| (x - 1, y - 1))
            .map_err(Into::into)
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> error::Result<()> {
        write!(self.buffer, "{}", cursor::Goto(x + 1, y + 1)).map_err(Into::into)
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
        match direction {
            MoveDirection::Up(n) => write!(self.buffer, "{}", cursor::Up(n))?,
            MoveDirection::Down(n) => write!(self.buffer, "{}", cursor::Down(n))?,
            MoveDirection::Left(n) => write!(self.buffer, "{}", cursor::Left(n))?,
            MoveDirection::Right(n) => write!(self.buffer, "{}", cursor::Right(n))?,
            _ => super::default_move_cursor(self, direction)?,
        }

        Ok(())
    }

    fn scroll(&mut self, dist: i16) -> error::Result<()> {
        match dist.cmp(&0) {
            Ordering::Greater => {
                write!(self.buffer, "{}", scroll::Down(dist as u16))
            }
            Ordering::Less => {
                write!(self.buffer, "{}", scroll::Up(-dist as u16))
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
        write!(self.buffer, "{}", Fg(color)).map_err(Into::into)
    }

    fn set_bg(&mut self, color: Color) -> error::Result<()> {
        write!(self.buffer, "{}", Bg(color)).map_err(Into::into)
    }

    fn clear(&mut self, clear_type: ClearType) -> error::Result<()> {
        match clear_type {
            ClearType::All => write!(self.buffer, "{}", clear::All),
            ClearType::FromCursorDown => {
                write!(self.buffer, "{}", clear::AfterCursor)
            }
            ClearType::FromCursorUp => {
                write!(self.buffer, "{}", clear::BeforeCursor)
            }
            ClearType::CurrentLine => write!(self.buffer, "{}", clear::CurrentLine),
            ClearType::UntilNewLine => {
                write!(self.buffer, "{}", clear::UntilNewline)
            }
        }
        .map_err(Into::into)
    }

    fn size(&self) -> error::Result<Size> {
        termion::terminal_size().map(Into::into).map_err(Into::into)
    }
}

pub(super) struct Fg(pub Color);

pub(super) struct Bg(pub Color);

impl fmt::Display for Fg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use color::Color as TermionColor;
        match self.0 {
            Color::Reset => color::Reset.write_fg(f),
            Color::Black => color::Black.write_fg(f),
            Color::Red => color::Red.write_fg(f),
            Color::Green => color::Green.write_fg(f),
            Color::Yellow => color::Yellow.write_fg(f),
            Color::Blue => color::Blue.write_fg(f),
            Color::Magenta => color::Magenta.write_fg(f),
            Color::Cyan => color::Cyan.write_fg(f),
            Color::Grey => color::White.write_fg(f),
            Color::DarkGrey => color::LightBlack.write_fg(f),
            Color::LightRed => color::LightRed.write_fg(f),
            Color::LightGreen => color::LightGreen.write_fg(f),
            Color::LightBlue => color::LightBlue.write_fg(f),
            Color::LightYellow => color::LightYellow.write_fg(f),
            Color::LightMagenta => color::LightMagenta.write_fg(f),
            Color::LightCyan => color::LightCyan.write_fg(f),
            Color::White => color::LightWhite.write_fg(f),
            Color::Ansi(i) => color::AnsiValue(i).write_fg(f),
            Color::Rgb(r, g, b) => color::Rgb(r, g, b).write_fg(f),
        }
    }
}
impl fmt::Display for Bg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use color::Color as TermionColor;
        match self.0 {
            Color::Reset => color::Reset.write_bg(f),
            Color::Black => color::Black.write_bg(f),
            Color::Red => color::Red.write_bg(f),
            Color::Green => color::Green.write_bg(f),
            Color::Yellow => color::Yellow.write_bg(f),
            Color::Blue => color::Blue.write_bg(f),
            Color::Magenta => color::Magenta.write_bg(f),
            Color::Cyan => color::Cyan.write_bg(f),
            Color::Grey => color::White.write_bg(f),
            Color::DarkGrey => color::LightBlack.write_bg(f),
            Color::LightRed => color::LightRed.write_bg(f),
            Color::LightGreen => color::LightGreen.write_bg(f),
            Color::LightBlue => color::LightBlue.write_bg(f),
            Color::LightYellow => color::LightYellow.write_bg(f),
            Color::LightMagenta => color::LightMagenta.write_bg(f),
            Color::LightCyan => color::LightCyan.write_bg(f),
            Color::White => color::LightWhite.write_bg(f),
            Color::Ansi(i) => color::AnsiValue(i).write_bg(f),
            Color::Rgb(r, g, b) => color::Rgb(r, g, b).write_bg(f),
        }
    }
}

pub(super) fn set_attributes<W: Write>(
    attributes: Attributes,
    mut w: W,
) -> error::Result<()> {
    if attributes.contains(Attributes::RESET) {
        write!(w, "{}", style::Reset)?;
    }

    if attributes.contains(Attributes::REVERSED) {
        write!(w, "{}", style::Invert)?;
    }
    if attributes.contains(Attributes::BOLD) {
        write!(w, "{}", style::Bold)?;
    }
    if attributes.contains(Attributes::ITALIC) {
        write!(w, "{}", style::Italic)?;
    }
    if attributes.contains(Attributes::UNDERLINED) {
        write!(w, "{}", style::Underline)?;
    }
    if attributes.contains(Attributes::DIM) {
        write!(w, "{}", style::Faint)?;
    }
    if attributes.contains(Attributes::CROSSED_OUT) {
        write!(w, "{}", style::CrossedOut)?;
    }
    if attributes.contains(Attributes::SLOW_BLINK)
        || attributes.contains(Attributes::RAPID_BLINK)
    {
        write!(w, "{}", style::Blink)?;
    }

    Ok(())
}

pub(super) fn remove_attributes<W: Write>(
    attributes: Attributes,
    mut w: W,
) -> error::Result<()> {
    if attributes.contains(Attributes::RESET) {
        write!(w, "{}", style::Reset)?;
    }

    if attributes.contains(Attributes::REVERSED) {
        write!(w, "{}", style::NoInvert)?;
    }
    if attributes.contains(Attributes::BOLD) {
        write!(w, "{}", style::NoBold)?;
    }
    if attributes.contains(Attributes::ITALIC) {
        write!(w, "{}", style::NoItalic)?;
    }
    if attributes.contains(Attributes::UNDERLINED) {
        write!(w, "{}", style::NoUnderline)?;
    }
    if attributes.contains(Attributes::DIM) {
        write!(w, "{}", style::NoFaint)?;
    }
    if attributes.contains(Attributes::CROSSED_OUT) {
        write!(w, "{}", style::NoCrossedOut)?;
    }
    if attributes.contains(Attributes::SLOW_BLINK)
        || attributes.contains(Attributes::RAPID_BLINK)
    {
        write!(w, "{}", style::NoBlink)?;
    }

    Ok(())
}

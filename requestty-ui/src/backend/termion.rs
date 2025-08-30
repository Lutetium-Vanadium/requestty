use std::{
    cmp::Ordering,
    fmt,
    io::{self, Write},
    ops::{Deref, DerefMut},
    os::fd::AsFd,
};

use termion::{
    clear, color, cursor,
    raw::{IntoRawMode, RawTerminal},
    scroll, style,
};

use super::{Attributes, Backend, ClearType, Color, DisplayBackend, MoveDirection, Size};

enum Terminal<W: Write + AsFd> {
    Raw(RawTerminal<W>),
    Normal(W),
    TemporaryNone,
}

impl<W: Write + AsFd> Deref for Terminal<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        match self {
            Terminal::Raw(w) => w,
            Terminal::Normal(w) => w,
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }
}

impl<W: Write + AsFd> DerefMut for Terminal<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Terminal::Raw(w) => w,
            Terminal::Normal(w) => w,
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }
}

/// A display backend that uses the `termion` library.
///
/// This is separate from [`TermionBackend`] to allow use without [`AsFd`] types.
#[allow(missing_debug_implementations)]
#[cfg_attr(docsrs, doc(cfg(feature = "termion")))]
pub struct TermionDisplayBackend<W: Write> {
    attributes: Attributes,
    buffer: W,
}

impl<W: Write> TermionDisplayBackend<W> {
    /// Creates a new [`TermionDisplayBackend`]
    pub fn new(buffer: W) -> Self {
        Self {
            buffer,
            attributes: Attributes::empty(),
        }
    }
}

impl<W: Write> Write for TermionDisplayBackend<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl<W: Write + AsFd> AsFd for TermionDisplayBackend<W> {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.buffer.as_fd()
    }
}

impl<W: Write> DisplayBackend for TermionDisplayBackend<W> {
    fn set_attributes(&mut self, attributes: Attributes) -> io::Result<()> {
        set_attributes(self.attributes, attributes, &mut self.buffer)?;
        self.attributes = attributes;
        Ok(())
    }

    fn set_fg(&mut self, color: Color) -> io::Result<()> {
        write!(self.buffer, "{}", Fg(color))
    }

    fn set_bg(&mut self, color: Color) -> io::Result<()> {
        write!(self.buffer, "{}", Bg(color))
    }
}

/// A backend that uses the `termion` library.
#[allow(missing_debug_implementations)]
#[cfg_attr(docsrs, doc(cfg(feature = "termion")))]
pub struct TermionBackend<W: Write + AsFd> {
    buffer: Terminal<TermionDisplayBackend<W>>,
}

impl<W: Write + AsFd> TermionBackend<W> {
    /// Creates a new [`TermionBackend`]
    pub fn new(buffer: W) -> Self {
        Self {
            buffer: Terminal::Normal(TermionDisplayBackend::new(buffer)),
        }
    }
}

impl<W: Write + AsFd> Write for TermionBackend<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl<W: Write + AsFd> DisplayBackend for TermionBackend<W> {
    fn set_attributes(&mut self, attributes: Attributes) -> io::Result<()> {
        match self.buffer {
            Terminal::Raw(ref mut buf) => buf.set_attributes(attributes),
            Terminal::Normal(ref mut buf) => buf.set_attributes(attributes),
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }

    fn set_fg(&mut self, color: Color) -> io::Result<()> {
        match self.buffer {
            Terminal::Raw(ref mut buf) => buf.set_fg(color),
            Terminal::Normal(ref mut buf) => buf.set_fg(color),
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }

    fn set_bg(&mut self, color: Color) -> io::Result<()> {
        match self.buffer {
            Terminal::Raw(ref mut buf) => buf.set_bg(color),
            Terminal::Normal(ref mut buf) => buf.set_bg(color),
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }

    fn write_styled(
        &mut self,
        styled: &crate::style::Styled<dyn fmt::Display + '_>,
    ) -> io::Result<()> {
        match self.buffer {
            Terminal::Raw(ref mut buf) => buf.write_styled(styled),
            Terminal::Normal(ref mut buf) => buf.write_styled(styled),
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }
}

impl<W: Write + AsFd> Backend for TermionBackend<W> {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        match self.buffer {
            Terminal::Raw(ref mut buf) => buf.activate_raw_mode(),
            Terminal::Normal(_) => {
                let buf = match std::mem::replace(&mut self.buffer, Terminal::TemporaryNone) {
                    Terminal::Normal(buf) => buf,
                    _ => unreachable!(),
                };

                self.buffer = Terminal::Raw(buf.into_raw_mode()?);

                Ok(())
            }
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        match self.buffer {
            Terminal::Raw(ref buf) => buf.suspend_raw_mode(),
            Terminal::Normal(_) => {
                if cfg!(debug_assertions) {
                    panic!("Called disable_raw_mode without enable_raw_mode");
                }

                Ok(())
            }
            Terminal::TemporaryNone => unreachable!("TemporaryNone is only used during swap"),
        }
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.buffer, "{}", cursor::Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.buffer, "{}", cursor::Show)
    }

    fn get_cursor_pos(&mut self) -> io::Result<(u16, u16)> {
        cursor::DetectCursorPos::cursor_pos(&mut *self.buffer)
            // 0 index the position
            .map(|(x, y)| (x - 1, y - 1))
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(self.buffer, "{}", cursor::Goto(x + 1, y + 1))
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> io::Result<()> {
        match direction {
            MoveDirection::Up(n) => write!(self.buffer, "{}", cursor::Up(n))?,
            MoveDirection::Down(n) => write!(self.buffer, "{}", cursor::Down(n))?,
            MoveDirection::Left(n) => write!(self.buffer, "{}", cursor::Left(n))?,
            MoveDirection::Right(n) => write!(self.buffer, "{}", cursor::Right(n))?,
            _ => super::default_move_cursor(self, direction)?,
        }

        Ok(())
    }

    fn scroll(&mut self, dist: i16) -> io::Result<()> {
        match dist.cmp(&0) {
            Ordering::Greater => {
                write!(self.buffer, "{}", scroll::Down(dist as u16))
            }
            Ordering::Less => {
                write!(self.buffer, "{}", scroll::Up(-dist as u16))
            }
            Ordering::Equal => Ok(()),
        }
    }

    fn clear(&mut self, clear_type: ClearType) -> io::Result<()> {
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
    }

    fn size(&self) -> io::Result<Size> {
        termion::terminal_size().map(Into::into)
    }
}

pub(super) struct Fg(pub(super) Color);

pub(super) struct Bg(pub(super) Color);

impl fmt::Display for Fg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    from: Attributes,
    to: Attributes,
    mut w: W,
) -> io::Result<()> {
    let diff = from.diff(to);

    if diff.to_remove.contains(Attributes::REVERSED) {
        write!(w, "{}", style::NoInvert)?;
    }
    if diff.to_remove.contains(Attributes::BOLD) {
        // XXX: the termion NoBold flag actually enables double-underline on ECMA-48 compliant
        // terminals, and NoFaint additionally disables bold... so we use this trick to get
        // the right semantics.
        write!(w, "{}", style::NoFaint)?;

        if to.contains(Attributes::DIM) {
            write!(w, "{}", style::Faint)?;
        }
    }
    if diff.to_remove.contains(Attributes::ITALIC) {
        write!(w, "{}", style::NoItalic)?;
    }
    if diff.to_remove.contains(Attributes::UNDERLINED) {
        write!(w, "{}", style::NoUnderline)?;
    }
    if diff.to_remove.contains(Attributes::DIM) {
        write!(w, "{}", style::NoFaint)?;

        // XXX: the NoFaint flag additionally disables bold as well, so we need to re-enable it
        // here if we want it.
        if to.contains(Attributes::BOLD) {
            write!(w, "{}", style::Bold)?;
        }
    }
    if diff.to_remove.contains(Attributes::CROSSED_OUT) {
        write!(w, "{}", style::NoCrossedOut)?;
    }
    if diff.to_remove.contains(Attributes::SLOW_BLINK)
        || diff.to_remove.contains(Attributes::RAPID_BLINK)
    {
        write!(w, "{}", style::NoBlink)?;
    }

    if diff.to_add.contains(Attributes::REVERSED) {
        write!(w, "{}", style::Invert)?;
    }
    if diff.to_add.contains(Attributes::BOLD) {
        write!(w, "{}", style::Bold)?;
    }
    if diff.to_add.contains(Attributes::ITALIC) {
        write!(w, "{}", style::Italic)?;
    }
    if diff.to_add.contains(Attributes::UNDERLINED) {
        write!(w, "{}", style::Underline)?;
    }
    if diff.to_add.contains(Attributes::DIM) {
        write!(w, "{}", style::Faint)?;
    }
    if diff.to_add.contains(Attributes::CROSSED_OUT) {
        write!(w, "{}", style::CrossedOut)?;
    }
    if diff.to_add.contains(Attributes::SLOW_BLINK) || diff.to_add.contains(Attributes::RAPID_BLINK)
    {
        write!(w, "{}", style::Blink)?;
    }

    Ok(())
}

use std::fmt::Display;

use crate::error;

pub struct Styled<T: ?Sized> {
    fg: Option<Color>,
    bg: Option<Color>,
    attributes: Attributes,
    content: T,
}

impl<T: Display> Styled<T> {
    pub fn new(content: T) -> Self {
        Self {
            fg: None,
            bg: None,
            attributes: Attributes::empty(),
            content,
        }
    }
}

impl<T: Display + ?Sized> Styled<T> {
    pub(super) fn write<B: crate::backend::Backend + ?Sized>(
        &self,
        backend: &mut B,
    ) -> error::Result<()> {
        if let Some(fg) = self.fg {
            backend.set_fg(fg)?;
        }
        if let Some(bg) = self.bg {
            backend.set_bg(bg)?;
        }
        if !self.attributes.is_empty() {
            backend.set_attributes(self.attributes)?;
        }

        write!(backend, "{}", &self.content)?;

        if self.fg.is_some() {
            backend.set_fg(Color::Reset)?;
        }
        if self.bg.is_some() {
            backend.set_bg(Color::Reset)?;
        }
        if !self.attributes.is_empty() {
            backend.set_attributes(Attributes::RESET)?;
        }
        Ok(())
    }
}

impl<T: Display> From<T> for Styled<T> {
    fn from(content: T) -> Self {
        Self::new(content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Grey,
    DarkGrey,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Ansi(u8),
}

bitflags::bitflags! {
    /// Attributes change the way a piece of text is displayed.
    pub struct Attributes: u16 {
        const RESET             = 0b0000_0000_0001;
        const BOLD              = 0b0000_0000_0010;
        const DIM               = 0b0000_0000_0100;
        const ITALIC            = 0b0000_0000_1000;
        const UNDERLINED        = 0b0000_0001_0000;
        const SLOW_BLINK        = 0b0000_0010_0000;
        const RAPID_BLINK       = 0b0000_0100_0000;
        const REVERSED          = 0b0000_1000_0000;
        const HIDDEN            = 0b0001_0000_0000;
        const CROSSED_OUT       = 0b0010_0000_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeDiff {
    pub to_add: Attributes,
    pub to_remove: Attributes,
}

impl Attributes {
    pub fn diff(self, other: Attributes) -> AttributeDiff {
        let diff = self ^ other;
        AttributeDiff {
            to_remove: diff & self,
            to_add: diff & other,
        }
    }
}

/// Provides a set of methods to set the colors and attributes.
///
/// Every method with the `on_` prefix sets the background color. Other color methods set the
/// foreground color. Method names correspond to the [`Attributes`] names.
///
/// Method names correspond to the [`Color`](enum.Color.html) enum variants.
pub trait Stylize<T> {
    fn black(self) -> Styled<T>;
    fn dark_grey(self) -> Styled<T>;
    fn light_red(self) -> Styled<T>;
    fn red(self) -> Styled<T>;
    fn light_green(self) -> Styled<T>;
    fn green(self) -> Styled<T>;
    fn light_yellow(self) -> Styled<T>;
    fn yellow(self) -> Styled<T>;
    fn light_blue(self) -> Styled<T>;
    fn blue(self) -> Styled<T>;
    fn light_magenta(self) -> Styled<T>;
    fn magenta(self) -> Styled<T>;
    fn light_cyan(self) -> Styled<T>;
    fn cyan(self) -> Styled<T>;
    fn white(self) -> Styled<T>;
    fn grey(self) -> Styled<T>;
    fn rgb(self, r: u8, g: u8, b: u8) -> Styled<T>;
    fn ansi(self, ansi: u8) -> Styled<T>;

    fn on_black(self) -> Styled<T>;
    fn on_dark_grey(self) -> Styled<T>;
    fn on_light_red(self) -> Styled<T>;
    fn on_red(self) -> Styled<T>;
    fn on_light_green(self) -> Styled<T>;
    fn on_green(self) -> Styled<T>;
    fn on_light_yellow(self) -> Styled<T>;
    fn on_yellow(self) -> Styled<T>;
    fn on_light_blue(self) -> Styled<T>;
    fn on_blue(self) -> Styled<T>;
    fn on_light_magenta(self) -> Styled<T>;
    fn on_magenta(self) -> Styled<T>;
    fn on_light_cyan(self) -> Styled<T>;
    fn on_cyan(self) -> Styled<T>;
    fn on_white(self) -> Styled<T>;
    fn on_grey(self) -> Styled<T>;
    fn on_rgb(self, r: u8, g: u8, b: u8) -> Styled<T>;
    fn on_ansi(self, ansi: u8) -> Styled<T>;

    fn reset(self) -> Styled<T>;
    fn bold(self) -> Styled<T>;
    fn underlined(self) -> Styled<T>;
    fn reverse(self) -> Styled<T>;
    fn dim(self) -> Styled<T>;
    fn italic(self) -> Styled<T>;
    fn slow_blink(self) -> Styled<T>;
    fn rapid_blink(self) -> Styled<T>;
    fn hidden(self) -> Styled<T>;
    fn crossed_out(self) -> Styled<T>;
}

impl<T, I: Into<Styled<T>>> Stylize<T> for I {
    fn black(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Black);
        styled
    }
    fn dark_grey(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::DarkGrey);
        styled
    }
    fn light_red(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::LightRed);
        styled
    }
    fn red(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Red);
        styled
    }
    fn light_green(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::LightGreen);
        styled
    }
    fn green(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Green);
        styled
    }
    fn light_yellow(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::LightYellow);
        styled
    }
    fn yellow(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Yellow);
        styled
    }
    fn light_blue(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::LightBlue);
        styled
    }
    fn blue(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Blue);
        styled
    }
    fn light_magenta(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::LightMagenta);
        styled
    }
    fn magenta(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Magenta);
        styled
    }
    fn light_cyan(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::LightCyan);
        styled
    }
    fn cyan(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Cyan);
        styled
    }
    fn white(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::White);
        styled
    }
    fn grey(self) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Grey);
        styled
    }
    fn rgb(self, r: u8, g: u8, b: u8) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Rgb(r, g, b));
        styled
    }
    fn ansi(self, ansi: u8) -> Styled<T> {
        let mut styled = self.into();
        styled.fg = Some(Color::Ansi(ansi));
        styled
    }

    fn on_black(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Black);
        styled
    }
    fn on_dark_grey(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::DarkGrey);
        styled
    }
    fn on_light_red(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::LightRed);
        styled
    }
    fn on_red(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Red);
        styled
    }
    fn on_light_green(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::LightGreen);
        styled
    }
    fn on_green(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Green);
        styled
    }
    fn on_light_yellow(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::LightYellow);
        styled
    }
    fn on_yellow(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Yellow);
        styled
    }
    fn on_light_blue(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::LightBlue);
        styled
    }
    fn on_blue(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Blue);
        styled
    }
    fn on_light_magenta(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::LightMagenta);
        styled
    }
    fn on_magenta(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Magenta);
        styled
    }
    fn on_light_cyan(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::LightCyan);
        styled
    }
    fn on_cyan(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Cyan);
        styled
    }
    fn on_white(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::White);
        styled
    }
    fn on_grey(self) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Grey);
        styled
    }
    fn on_rgb(self, r: u8, g: u8, b: u8) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Rgb(r, g, b));
        styled
    }
    fn on_ansi(self, ansi: u8) -> Styled<T> {
        let mut styled = self.into();
        styled.bg = Some(Color::Ansi(ansi));
        styled
    }

    fn reset(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::RESET;
        styled
    }
    fn bold(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::BOLD;
        styled
    }
    fn underlined(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::UNDERLINED;
        styled
    }
    fn reverse(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::REVERSED;
        styled
    }
    fn dim(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::DIM;
        styled
    }
    fn italic(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::ITALIC;
        styled
    }
    fn slow_blink(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::SLOW_BLINK;
        styled
    }
    fn rapid_blink(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::RAPID_BLINK;
        styled
    }
    fn hidden(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::HIDDEN;
        styled
    }
    fn crossed_out(self) -> Styled<T> {
        let mut styled = self.into();
        styled.attributes |= Attributes::CROSSED_OUT;
        styled
    }
}

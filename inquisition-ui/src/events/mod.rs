#[cfg(feature = "termion")]
use std::io::{stdin, Stdin};

use crate::error;

crate::cfg_async! {
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::AsyncEvents;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::AsyncEvents;
}

#[cfg(feature = "crossterm")]
mod crossterm;
#[cfg(feature = "termion")]
mod termion;

bitflags::bitflags! {
    /// Represents key modifiers (shift, control, alt).
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
    }
}

/// Represents a key event.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct KeyEvent {
    /// The key itself.
    pub code: KeyCode,
    /// Additional key modifiers.
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent { code, modifiers }
    }
}

impl From<KeyCode> for KeyEvent {
    fn from(code: KeyCode) -> Self {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
        }
    }
}

/// Represents a key.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyCode {
    /// Backspace key.
    Backspace,
    /// Enter key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page dow key.
    PageDown,
    /// Tab key.
    Tab,
    /// Shift + Tab key.
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// F key.
    ///
    /// `KeyEvent::F(1)` represents F1 key, etc.
    F(u8),
    /// A character.
    ///
    /// `KeyEvent::Char('c')` represents `c` character, etc.
    Char(char),
    /// Null.
    Null,
    /// Escape key.
    Esc,
}

/// Movements that can be captured from a [`KeyEvent`]. See the individual variants for
/// what keys they capture
pub enum Movement {
    /// The Up arrow key, and `k` is captured
    Up,
    /// The Down arrow key, and `j` is captured
    Down,
    /// The Left arrow key, `h`, and `ctrl+b` is captured
    Left,
    /// The Right arrow key, `l`, and `ctrl+f` is captured
    Right,
    /// The PageUp key is captured
    PageUp,
    /// The PageDown key is captured
    PageDown,
    /// The Home key, `g`, `ctrl+a`  is captured
    Home,
    /// The End key, `G`, `ctrl+e`  is captured
    End,
    /// `ctrl+right`, `alt+right`, and `alt+f` are captured
    NextWord,
    /// `ctrl+left`, `alt+left`, and `alt+b` are captured
    PrevWord,
}

impl Movement {
    /// Gets the movement (if any) from the current event
    ///
    /// note: it also captures 'h', 'j', 'k', 'l', 'g', and 'G'. If these are required
    /// for some input, it must be checked before capturing a movement
    pub fn try_from_key(key: KeyEvent) -> Option<Movement> {
        let movement = match key.code {
            KeyCode::Left
                if key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Movement::PrevWord
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                Movement::PrevWord
            }

            KeyCode::Right
                if key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Movement::NextWord
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                Movement::NextWord
            }

            KeyCode::Up => Movement::Up,
            KeyCode::Char('k') => Movement::Up,

            KeyCode::Down => Movement::Down,
            KeyCode::Char('j') => Movement::Down,

            KeyCode::Left => Movement::Left,
            KeyCode::Char('h') => Movement::Left,
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::Left
            }

            KeyCode::Right => Movement::Right,
            KeyCode::Char('l') => Movement::Right,
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::Right
            }

            KeyCode::PageUp => Movement::PageUp,
            KeyCode::PageDown => Movement::PageDown,

            KeyCode::Home => Movement::Home,
            KeyCode::Char('g') => Movement::Home,
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::Home
            }

            KeyCode::End => Movement::End,
            KeyCode::Char('G') => Movement::End,
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Movement::End
            }
            _ => return None,
        };

        Some(movement)
    }
}

pub struct Events {
    #[cfg(feature = "termion")]
    events: ::termion::input::Keys<Stdin>,
}

impl Events {
    #[cfg(feature = "termion")]
    pub fn new() -> Self {
        #[rustfmt::skip]
        use ::termion::input::TermRead;

        Self {
            events: stdin().keys(),
        }
    }

    #[cfg(not(feature = "termion"))]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Events {
    type Item = error::Result<KeyEvent>;

    #[cfg(feature = "crossterm")]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self::crossterm::next_event())
    }

    #[cfg(feature = "termion")]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self::termion::next_event(&mut self.events))
    }
}

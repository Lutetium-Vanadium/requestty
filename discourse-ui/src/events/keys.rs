bitflags::bitflags! {
    /// Represents key modifiers (shift, control, alt).
    pub struct KeyModifiers: u8 {
        #[allow(missing_docs)]
        const SHIFT = 0b0000_0001;
        #[allow(missing_docs)]
        const CONTROL = 0b0000_0010;
        #[allow(missing_docs)]
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
    /// Creates a new `KeyEvent`
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
    /// A function key.
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

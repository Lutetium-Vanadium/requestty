use crossterm::event;

use crate::error;

pub fn next_event() -> error::Result<super::KeyEvent> {
    loop {
        if let event::Event::Key(k) = event::read()? {
            return Ok(k.into());
        }
    }
}

impl From<event::KeyEvent> for super::KeyEvent {
    fn from(event: event::KeyEvent) -> Self {
        let code = match event.code {
            event::KeyCode::Backspace => super::KeyCode::Backspace,
            event::KeyCode::Enter => super::KeyCode::Enter,
            event::KeyCode::Left => super::KeyCode::Left,
            event::KeyCode::Right => super::KeyCode::Right,
            event::KeyCode::Up => super::KeyCode::Up,
            event::KeyCode::Down => super::KeyCode::Down,
            event::KeyCode::Home => super::KeyCode::Home,
            event::KeyCode::End => super::KeyCode::End,
            event::KeyCode::PageUp => super::KeyCode::PageUp,
            event::KeyCode::PageDown => super::KeyCode::PageDown,
            event::KeyCode::Tab => super::KeyCode::Tab,
            event::KeyCode::BackTab => super::KeyCode::BackTab,
            event::KeyCode::Delete => super::KeyCode::Delete,
            event::KeyCode::Insert => super::KeyCode::Insert,
            event::KeyCode::F(f) => super::KeyCode::F(f),
            event::KeyCode::Char(c) => super::KeyCode::Char(c),
            event::KeyCode::Null => super::KeyCode::Null,
            event::KeyCode::Esc => super::KeyCode::Esc,
        };

        let mut modifiers = super::KeyModifiers::empty();

        if event.modifiers.contains(event::KeyModifiers::SHIFT) {
            modifiers |= super::KeyModifiers::SHIFT;
        }
        if event.modifiers.contains(event::KeyModifiers::CONTROL) {
            modifiers |= super::KeyModifiers::CONTROL;
        }
        if event.modifiers.contains(event::KeyModifiers::ALT) {
            modifiers |= super::KeyModifiers::ALT;
        }

        super::KeyEvent { code, modifiers }
    }
}

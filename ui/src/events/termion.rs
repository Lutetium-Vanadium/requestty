use std::{
    convert::{TryFrom, TryInto},
    io::{self, Stdin},
};

use termion::{event, input};

use crate::error;

pub fn next_event(
    events: &mut input::Keys<Stdin>,
) -> error::Result<super::KeyEvent> {
    let e = events.next().unwrap()?;
    e.try_into()
}

impl TryFrom<event::Key> for super::KeyEvent {
    type Error = error::ErrorKind;

    fn try_from(key: event::Key) -> error::Result<super::KeyEvent> {
        let key = match key {
            event::Key::Backspace => super::KeyCode::Backspace.into(),
            event::Key::Left => super::KeyCode::Left.into(),
            event::Key::Right => super::KeyCode::Right.into(),
            event::Key::Up => super::KeyCode::Up.into(),
            event::Key::Down => super::KeyCode::Down.into(),
            event::Key::Home => super::KeyCode::Home.into(),
            event::Key::End => super::KeyCode::End.into(),
            event::Key::PageUp => super::KeyCode::PageUp.into(),
            event::Key::PageDown => super::KeyCode::PageDown.into(),
            event::Key::BackTab => super::KeyCode::BackTab.into(),
            event::Key::Delete => super::KeyCode::Delete.into(),
            event::Key::Insert => super::KeyCode::Insert.into(),
            event::Key::F(n) => super::KeyCode::F(n).into(),
            event::Key::Char('\n') => super::KeyCode::Enter.into(),
            event::Key::Char('\t') => super::KeyCode::Tab.into(),
            event::Key::Char(c) => super::KeyCode::Char(c).into(),
            event::Key::Alt(c) => parse_char(c, super::KeyModifiers::ALT)?,
            event::Key::Ctrl(c) => parse_char(c, super::KeyModifiers::CONTROL)?,
            event::Key::Null => super::KeyCode::Null.into(),
            event::Key::Esc => super::KeyCode::Esc.into(),
            _ => unreachable!(),
        };

        Ok(key)
    }
}

fn parse_char(
    mut c: char,
    mut modifiers: super::KeyModifiers,
) -> error::Result<super::KeyEvent> {
    let code = loop {
        if c as u32 >= 256 {
            break super::KeyCode::Char(c);
        }

        let k = match event::parse_event(c as u8, &mut std::iter::empty())? {
            event::Event::Key(k) => k,
            _ => match char::try_from(c as u32) {
                Ok(c) => break super::KeyCode::Char(c),
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Could not parse an event",
                    )
                    .into())
                }
            },
        };

        match k {
            event::Key::Backspace => break super::KeyCode::Backspace,
            event::Key::Left => break super::KeyCode::Left,
            event::Key::Right => break super::KeyCode::Right,
            event::Key::Up => break super::KeyCode::Up,
            event::Key::Down => break super::KeyCode::Down,
            event::Key::Home => break super::KeyCode::Home,
            event::Key::End => break super::KeyCode::End,
            event::Key::PageUp => break super::KeyCode::PageUp,
            event::Key::PageDown => break super::KeyCode::PageDown,
            event::Key::BackTab => break super::KeyCode::BackTab,
            event::Key::Delete => break super::KeyCode::Delete,
            event::Key::Insert => break super::KeyCode::Insert,
            event::Key::F(n) => break super::KeyCode::F(n),
            event::Key::Char('\n') => break super::KeyCode::Enter,
            event::Key::Char('\t') => break super::KeyCode::Tab,
            event::Key::Char(c) => break super::KeyCode::Char(c),
            event::Key::Alt(new_c) => {
                modifiers |= super::KeyModifiers::ALT;
                c = new_c
            }
            event::Key::Ctrl(new_c) => {
                modifiers |= super::KeyModifiers::CONTROL;
                c = new_c
            }
            event::Key::Null => break super::KeyCode::Null,
            event::Key::Esc => break super::KeyCode::Esc,
            _ => unreachable!(),
        }
    };

    Ok(super::KeyEvent::new(code, modifiers))
}

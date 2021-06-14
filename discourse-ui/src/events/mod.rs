#[cfg(feature = "termion")]
use std::io::{stdin, Stdin};

use crate::error;

#[cfg(feature = "crossterm")]
mod crossterm;
#[cfg(feature = "termion")]
mod termion;

mod keys;
mod movement;

pub use keys::{KeyCode, KeyEvent, KeyModifiers};
pub use movement::Movement;

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

pub struct TestEvents<E: Iterator<Item = KeyEvent>> {
    events: E,
}

impl<E: Iterator<Item = KeyEvent>> TestEvents<E> {
    pub fn new<I: IntoIterator<IntoIter = E, Item = KeyEvent>>(iter: I) -> Self {
        Self {
            events: iter.into_iter(),
        }
    }
}

impl<E: Iterator<Item = KeyEvent>> Iterator for TestEvents<E> {
    type Item = error::Result<KeyEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.events.next().map(Ok)
    }
}

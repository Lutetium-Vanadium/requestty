//! A module for handling key events

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

/// An iterator over the input keys
#[allow(missing_debug_implementations)]
pub struct Events {
    #[cfg(feature = "termion")]
    events: ::termion::input::Keys<Stdin>,
}

impl Events {
    #[cfg(feature = "termion")]
    /// Creates a new [`Events`] using stdin
    pub fn new() -> Self {
        #[rustfmt::skip]
        use ::termion::input::TermRead;

        Self {
            events: stdin().keys(),
        }
    }

    #[cfg(not(feature = "termion"))]
    /// Creates a new [`Events`]
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

    // `cargo doc` fails if this doesn't exist
    #[cfg(all(not(feature = "crossterm"), not(feature = "termion")))]
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

/// A simple wrapper around on a [`KeyEvent`] iterator that can be used in tests
#[derive(Debug, Clone)]
pub struct TestEvents<E> {
    events: E,
}

impl<E: Iterator<Item = KeyEvent>> TestEvents<E> {
    /// Create a new `TestEvents`
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

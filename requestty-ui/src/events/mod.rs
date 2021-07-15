//! A module for handling key events

use std::io;

#[cfg(feature = "crossterm")]
mod crossterm;
#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "crossterm")]
pub use self::crossterm::CrosstermEvents;

#[cfg(feature = "termion")]
pub use self::termion::TermionEvents;

mod keys;
mod movement;

pub use keys::{KeyCode, KeyEvent, KeyModifiers};
pub use movement::Movement;

/// Gets the default [`EventIterator`] based on the features enabled.
#[cfg(any(feature = "crossterm", feature = "termion"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
pub fn get_events() -> impl EventIterator {
    #[cfg(feature = "crossterm")]
    return CrosstermEvents::new();

    // XXX: Only works when crossterm and termion are the only two available backends
    //
    // Instead of directly checking for termion, we check for not crossterm so that compiling
    // (documentation) with both features enabled will not error
    #[cfg(not(feature = "crossterm"))]
    return TermionEvents::new();
}

/// A trait to represent a source of [`KeyEvent`]s.
pub trait EventIterator {
    /// Get the next event
    fn next_event(&mut self) -> io::Result<KeyEvent>;
}

/// A simple wrapper around a [`KeyEvent`] iterator that can be used in tests.
///
/// Even though [`EventIterator`] expects the iterator to be infinite, only having enough events to
/// complete the test is necessary.
///
/// It will also check that the internal iterator is fully exhausted on [`Drop`].
///
/// # Panics
///
/// It will panic if the events run out [`next_event`] is called, or if there are events remaining
/// when dropped.
///
/// [`next_event`]: TestEvents::next_event
#[derive(Debug, Clone)]
pub struct TestEvents<E: Iterator<Item = KeyEvent>> {
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

impl TestEvents<std::iter::Empty<KeyEvent>> {
    /// Create a new `TestEvents` which yields no events
    pub fn empty() -> Self {
        Self {
            events: std::iter::empty(),
        }
    }
}

impl<E: Iterator<Item = KeyEvent>> EventIterator for TestEvents<E> {
    fn next_event(&mut self) -> io::Result<KeyEvent> {
        Ok(self
            .events
            .next()
            .expect("Events ran out, but another one was requested"))
    }
}

impl<E: Iterator<Item = KeyEvent>> Drop for TestEvents<E> {
    fn drop(&mut self) {
        let mut count = 0;

        while self.events.next().is_some() {
            count += 1
        }

        assert_eq!(
            count, 0,
            "Events did not fully run out, {} events have not been consumed",
            count
        );
    }
}

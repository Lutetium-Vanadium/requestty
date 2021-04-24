#![deny(missing_docs, rust_2018_idioms)]
//! A widget based cli ui rendering library
use std::sync::Mutex;

use crossterm::terminal;

#[cfg(feature = "async")]
pub use async_input::{AsyncInput, AsyncPrompt};
pub use sync_input::{Input, Prompt};
pub use widget::Widget;

/// In build widgets
pub mod widgets {
    pub use crate::char_input::CharInput;
    pub use crate::list::{List, ListPicker};
    pub use crate::string_input::StringInput;

    /// The default type for filter_map_char in [`StringInput`] and [`CharInput`]
    pub type FilterMapChar = fn(char) -> Option<char>;

    /// Character filter that lets every character through
    pub fn no_filter(c: char) -> Option<char> {
        Some(c)
    }
}

#[cfg(feature = "async")]
mod async_input;
mod char_input;
mod list;
mod string_input;
mod sync_input;
mod widget;

/// Returned by [`Prompt::validate`]
pub enum Validation {
    /// If the prompt is ready to finish.
    Finish,
    /// If the state is valid, but the prompt should still persist.
    /// Unlike returning an Err, this will not print anything unique, and is a way for the prompt to
    /// say that it internally has processed the `Enter` key, but is not complete.
    Continue,
}

lazy_static::lazy_static! {
    static ref EXIT_HANDLER: Mutex<fn()> = Mutex::new(default_exit);
}

/// Sets the exit handler to call when `CTRL+C` or EOF is received
///
/// By default, it exits the program, however it can be overridden to not exit. If it doesn't exit,
/// [`Input::run`] will return an `Err`
pub fn set_exit_handler(handler: fn()) {
    *EXIT_HANDLER.lock().unwrap() = handler;
}

fn default_exit() {
    std::process::exit(130);
}

fn exit() {
    match EXIT_HANDLER.lock() {
        Ok(exit) => exit(),
        Err(_) => default_exit(),
    }
}

/// Simple helper to make sure if the code panics in between, raw mode is disabled
struct RawMode {
    _private: (),
}

impl RawMode {
    /// Enable raw mode for the terminal
    pub fn enable() -> crossterm::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self { _private: () })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

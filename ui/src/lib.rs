#![deny(missing_docs, rust_2018_idioms)]
//! A widget based cli ui rendering library
use std::{convert::TryFrom, io, sync::Mutex};

use crossterm::{
    cursor, event, execute, queue,
    style::{Colorize, Print, PrintStyledContent, Styler},
    terminal,
};

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

mod char_input;
mod list;
mod string_input;
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

/// This trait should be implemented by all 'root' widgets.
///
/// It provides the functionality specifically required only by the main controlling widget. For the
/// trait required for general rendering to terminal, see [`Widget`].
pub trait Prompt: Widget {
    /// The error type returned by validate. It **must** be only one line long.
    type ValidateErr: std::fmt::Display;

    /// The output type returned by [`Input::run`]
    type Output;

    /// The main prompt text. It is printed in bold.
    fn prompt(&self) -> &str;
    /// The hint text. If a hint is there, it is printed in dark grey.
    fn hint(&self) -> Option<&str> {
        None
    }

    /// Determine whether the prompt state is ready to be submitted. It is called whenever the use
    /// presses the enter key.
    #[allow(unused_variables)]
    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        Ok(Validation::Finish)
    }
    /// The value to return from [`Input::run`]. This will only be called after
    /// [`validate`](Prompt::validate), if validate returns `Ok(true)`
    fn finish(self) -> Self::Output;

    /// The prompt has some default value that can be returned.
    fn has_default(&self) -> bool;
    /// The default value to be returned. It will only be called when has_default is true and the
    /// user presses escape.
    fn finish_default(self) -> Self::Output
    where
        Self: Sized,
    {
        unreachable!();
    }
}

/// The ui runner. It renders and processes events with the help of a type that implements [`Prompt`]
///
/// See [`run`](Input::run) for more information
pub struct Input<P> {
    prompt: P,
    terminal_h: u16,
    terminal_w: u16,
    base_row: u16,
    base_col: u16,
    error_row: Option<u16>,
    hide_cursor: bool,
}

impl<P: Prompt> Input<P> {
    fn adjust_scrollback<W: io::Write>(
        &self,
        height: usize,
        stdout: &mut W,
    ) -> crossterm::Result<u16> {
        let th = self.terminal_h as usize;

        let mut base_row = self.base_row;

        if self.base_row as usize >= th - height {
            let dist = (self.base_row as usize + height - th + 1) as u16;
            base_row -= dist;
            queue!(stdout, terminal::ScrollUp(dist), cursor::MoveUp(dist))?;
        }

        Ok(base_row)
    }

    fn set_cursor_pos<W: io::Write>(&self, stdout: &mut W) -> crossterm::Result<()> {
        let (dcw, dch) = self.prompt.cursor_pos(self.base_col);
        execute!(stdout, cursor::MoveTo(dcw, self.base_row + dch))
    }

    fn render<W: io::Write>(&mut self, stdout: &mut W) -> crossterm::Result<()> {
        let height = self.prompt.height();
        self.base_row = self.adjust_scrollback(height, stdout)?;
        self.clear(self.base_col, stdout)?;
        queue!(stdout, cursor::MoveTo(self.base_col, self.base_row))?;

        self.prompt
            .render((self.terminal_w - self.base_col) as usize, stdout)?;

        self.set_cursor_pos(stdout)
    }

    fn clear<W: io::Write>(&self, prompt_len: u16, stdout: &mut W) -> crossterm::Result<()> {
        if self.base_row + 1 < self.terminal_h {
            queue!(
                stdout,
                cursor::MoveTo(0, self.base_row + 1),
                terminal::Clear(terminal::ClearType::FromCursorDown),
            )?;
        }

        queue!(
            stdout,
            cursor::MoveTo(prompt_len, self.base_row),
            terminal::Clear(terminal::ClearType::UntilNewLine),
        )
    }

    #[inline]
    fn finish<W: io::Write>(
        self,
        pressed_enter: bool,
        prompt_len: u16,
        stdout: &mut W,
    ) -> crossterm::Result<P::Output> {
        self.clear(prompt_len, stdout)?;
        stdout.flush()?;
        if pressed_enter {
            Ok(self.prompt.finish())
        } else {
            Ok(self.prompt.finish_default())
        }
    }

    /// Run the ui on the given writer. It will return when the user presses `Enter` or `Escape`
    /// based on the [`Prompt`] implementation.
    pub fn run<W: io::Write>(mut self, stdout: &mut W) -> crossterm::Result<P::Output> {
        let (tw, th) = terminal::size()?;
        self.terminal_h = th;
        self.terminal_w = tw;

        let prompt = self.prompt.prompt();
        let prompt_len = u16::try_from(prompt.chars().count() + 3).expect("really big prompt");

        let _raw = RawMode::enable()?;
        if self.hide_cursor {
            queue!(stdout, cursor::Hide)?;
        };

        let height = self.prompt.height();
        self.base_row = cursor::position()?.1;
        self.base_row = self.adjust_scrollback(height, stdout)?;

        queue!(
            stdout,
            PrintStyledContent("? ".green()),
            PrintStyledContent(prompt.bold()),
            Print(' '),
        )?;

        let hint_len = match self.prompt.hint() {
            Some(hint) => {
                queue!(stdout, PrintStyledContent(hint.dark_grey()), Print(' '))?;
                u16::try_from(hint.chars().count() + 1).expect("really big prompt")
            }
            None => 0,
        };

        self.base_col = prompt_len + hint_len;

        self.render(stdout)?;

        loop {
            match event::read()? {
                event::Event::Resize(tw, th) => {
                    self.terminal_w = tw;
                    self.terminal_h = th;
                }

                event::Event::Key(e) => {
                    if let Some(error_row) = self.error_row.take() {
                        let pos = cursor::position()?;
                        queue!(
                            stdout,
                            cursor::MoveTo(0, error_row),
                            terminal::Clear(terminal::ClearType::CurrentLine),
                            cursor::MoveTo(pos.0, pos.1)
                        )?;
                    }

                    let key_handled = match e.code {
                        event::KeyCode::Char('c')
                            if e.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            queue!(
                                stdout,
                                cursor::MoveTo(0, self.base_row + self.prompt.height() as u16)
                            )?;
                            drop(_raw);
                            if self.hide_cursor {
                                queue!(stdout, cursor::Show)?;
                            }
                            exit()
                        }
                        event::KeyCode::Null => {
                            queue!(
                                stdout,
                                cursor::MoveTo(0, self.base_row + self.prompt.height() as u16)
                            )?;
                            drop(_raw);
                            if self.hide_cursor {
                                queue!(stdout, cursor::Show)?;
                            }
                            exit()
                        }
                        event::KeyCode::Esc if self.prompt.has_default() => {
                            return self.finish(false, prompt_len, stdout);
                        }

                        event::KeyCode::Enter => match self.prompt.validate() {
                            Ok(Validation::Finish) => {
                                return self.finish(true, prompt_len, stdout);
                            }
                            Ok(Validation::Continue) => true,
                            Err(e) => {
                                let height = self.prompt.height() + 1;
                                self.base_row = self.adjust_scrollback(height, stdout)?;

                                queue!(stdout, cursor::MoveTo(0, self.base_row + height as u16))?;
                                write!(stdout, "{} {}", ">>".dark_red(), e)?;

                                self.error_row = Some(self.base_row + height as u16);

                                self.set_cursor_pos(stdout)?;

                                continue;
                            }
                        },
                        _ => self.prompt.handle_key(e),
                    };

                    if key_handled {
                        self.render(stdout)?;
                    }
                }
                _ => {}
            }
        }
    }
}

impl<P> Input<P> {
    /// Creates a new Input
    pub fn new(prompt: P) -> Self {
        Self {
            prompt,
            base_row: 0,
            base_col: 0,
            terminal_h: 0,
            terminal_w: 0,
            error_row: None,
            hide_cursor: false,
        }
    }

    /// Hides the cursor while running the input
    pub fn hide_cursor(mut self) -> Self {
        self.hide_cursor = true;
        self
    }
}

lazy_static::lazy_static! {
    static ref EXIT_HANDLER: Mutex<fn() -> !> = Mutex::new(default_exit);
}

/// Sets the exit handler to call when CTRL+C is received
pub fn set_exit_handler(handler: fn() -> !) {
    *EXIT_HANDLER.lock().unwrap() = handler;
}

fn default_exit() -> ! {
    std::process::exit(130);
}

fn exit() -> ! {
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

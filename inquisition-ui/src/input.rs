use std::convert::TryFrom;

use super::{TerminalState, Validation, Widget};
use crate::{
    backend::{Backend, ClearType, MoveDirection, Size, Stylize},
    error,
    events::{Events, KeyCode, KeyModifiers},
    Layout,
};

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
    /// The value to return from [`Input::run`]. This will only be called once validation returns
    /// [`Validation::Finish`];
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
pub struct Input<P, B: Backend> {
    pub(super) prompt: P,
    pub(super) backend: TerminalState<B>,
    pub(super) base_row: u16,
    pub(super) base_col: u16,
    pub(super) size: Size,
}

impl<P: Prompt, B: Backend> Input<P, B> {
    pub(super) fn layout(&self) -> Layout {
        Layout::new(self.base_col, self.size).with_offset(0, self.base_row)
    }

    pub(super) fn init(&mut self) -> error::Result<u16> {
        let prompt = self.prompt.prompt();
        let prompt_len =
            u16::try_from(prompt.chars().count() + 3).expect("really big prompt");

        self.size = self.backend.size()?;
        self.backend.init()?;

        self.backend.write_styled("? ".light_green())?;
        self.backend.write_styled(prompt.bold())?;
        self.backend.write_all(b" ")?;

        let hint_len = match self.prompt.hint() {
            Some(hint) => {
                self.backend.write_styled(hint.dark_grey())?;
                self.backend.write_all(b" ")?;
                u16::try_from(hint.chars().count() + 1).expect("really big prompt")
            }
            None => 0,
        };

        self.base_row = self.backend.get_cursor()?.1;
        let height = self.prompt.height(self.layout());
        self.base_row = self.adjust_scrollback(height)?;
        self.base_col = prompt_len + hint_len;

        self.render()?;

        Ok(prompt_len)
    }

    pub(super) fn adjust_scrollback(&mut self, height: u16) -> error::Result<u16> {
        let th = self.size.height;

        let mut base_row = self.base_row;

        if self.base_row >= th - height {
            let dist = self.base_row + height - th + 1;
            base_row -= dist;
            self.backend.scroll(-(dist as i32))?;
            self.backend.move_cursor(MoveDirection::Up(dist))?;
        }

        Ok(base_row)
    }

    pub(super) fn flush(&mut self) -> error::Result<()> {
        if !self.backend.hide_cursor {
            let (dcw, dch) = self.prompt.cursor_pos(self.layout());
            self.backend.set_cursor(dcw, self.base_row + dch)?;
        }
        self.backend.flush().map_err(Into::into)
    }

    pub(super) fn render(&mut self) -> error::Result<()> {
        let height = self.prompt.height(self.layout());
        self.base_row = self.adjust_scrollback(height)?;
        self.clear(self.base_col)?;
        self.backend.set_cursor(self.base_col, self.base_row)?;

        self.prompt.render(self.layout(), &mut *self.backend)?;

        self.flush()
    }

    pub(super) fn clear(&mut self, prompt_len: u16) -> error::Result<()> {
        if self.base_row + 1 < self.size.height {
            self.backend.set_cursor(0, self.base_row + 1)?;
            self.backend.clear(ClearType::FromCursorDown)?;
        }

        self.backend.set_cursor(prompt_len, self.base_row)?;
        self.backend.clear(ClearType::UntilNewLine)
    }

    pub(super) fn goto_last_line(&mut self) -> error::Result<()> {
        let height = self.prompt.height(self.layout()) + 1;
        self.base_row = self.adjust_scrollback(height)?;
        self.backend.set_cursor(0, self.base_row + height as u16)
    }

    pub(super) fn print_error(&mut self, e: P::ValidateErr) -> error::Result<()> {
        self.size = self.backend.size()?;
        self.goto_last_line()?;
        self.backend.write_styled(">>".red())?;
        write!(self.backend, " {}", e)?;
        self.flush()
    }

    pub(super) fn exit(&mut self) -> error::Result<()> {
        self.size = self.backend.size()?;
        self.goto_last_line()?;
        self.backend.reset()?;
        Ok(())
    }

    #[inline]
    fn finish(
        mut self,
        pressed_enter: bool,
        prompt_len: u16,
    ) -> error::Result<P::Output> {
        self.clear(prompt_len)?;
        self.backend.reset()?;

        if pressed_enter {
            Ok(self.prompt.finish())
        } else {
            Ok(self.prompt.finish_default())
        }
    }

    /// Run the ui on the given writer. It will return when the user presses `Enter` or `Escape`
    /// based on the [`Prompt`] implementation.
    pub fn run(mut self, events: &mut Events) -> error::Result<P::Output> {
        let prompt_len = self.init()?;

        loop {
            let e = events.next().unwrap()?;

            let key_handled = match e.code {
                KeyCode::Char('c')
                    if e.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.exit()?;
                    return Err(error::ErrorKind::Interrupted);
                }
                KeyCode::Null => {
                    self.exit()?;
                    return Err(error::ErrorKind::Eof);
                }
                KeyCode::Esc if self.prompt.has_default() => {
                    return self.finish(false, prompt_len);
                }

                KeyCode::Enter => match self.prompt.validate() {
                    Ok(Validation::Finish) => {
                        return self.finish(true, prompt_len);
                    }
                    Ok(Validation::Continue) => true,
                    Err(e) => {
                        self.print_error(e)?;

                        continue;
                    }
                },
                _ => self.prompt.handle_key(e),
            };

            if key_handled {
                self.size = self.backend.size()?;
                self.render()?;
            }
        }
    }
}

impl<P, B: Backend> Input<P, B> {
    #[allow(clippy::new_ret_no_self)]
    /// Creates a new Input
    pub fn new(prompt: P, backend: &mut B) -> Input<P, &mut B> {
        // The method doesn't return self directly, as its always used with a `&mut B`,
        // and this tells the compiler that it doesn't need to consume the `&mut B`, but
        // once the Input goes out of scope, it can be used again
        Input {
            prompt,
            backend: TerminalState::new(backend, false),
            base_row: 0,
            base_col: 0,
            size: Size::default(),
        }
    }

    /// Hides the cursor while running the input
    pub fn hide_cursor(mut self) -> Self {
        self.backend.hide_cursor = true;
        self
    }
}

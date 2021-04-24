use std::{convert::TryFrom, io};

use async_trait::async_trait;
use crossterm::{
    cursor, event, execute, queue,
    style::{Colorize, Print, PrintStyledContent, Styler},
    terminal,
};
use futures::StreamExt;

use crate::{Prompt, RawMode, Validation};

/// This trait should be implemented by all 'root' widgets.
///
/// It provides the functionality specifically required only by the main controlling widget. For the
/// trait required for general rendering to terminal, see [`Widget`].
#[async_trait]
pub trait AsyncPrompt: Prompt {
    /// Try to validate the prompt state is valid synchronously without blocking. If it can't be
    /// done without blocking, return `None` and [`validate_async`](AsyncPrompt::validate_async)
    /// will be called instead. It is called whenever the use presses the enter key.
    fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
        None
    }

    /// Determine whether the prompt state is ready to be submitted. It is called whenever the use
    /// presses the enter key.
    #[allow(unused_variables)]
    async fn validate_async(&mut self) -> Result<Validation, Self::ValidateErr> {
        Ok(Validation::Finish)
    }

    /// The value to return from [`AsyncInput::run`]. This will only be called once validation returns
    /// [`Validation::Finish`];
    async fn finish_async(self) -> Self::Output;
}

/// The ui runner. It renders and processes events with the help of a type that implements [`AsyncPrompt`]
///
/// See [`run`](AsyncInput::run) for more information
pub struct AsyncInput<P> {
    prompt: P,
    terminal_h: u16,
    terminal_w: u16,
    base_row: u16,
    base_col: u16,
    hide_cursor: bool,
}

impl<P: AsyncPrompt + Send> AsyncInput<P> {
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
    async fn finish<W: io::Write>(
        self,
        pressed_enter: bool,
        prompt_len: u16,
        stdout: &mut W,
    ) -> crossterm::Result<P::Output> {
        self.clear(prompt_len, stdout)?;
        if self.hide_cursor {
            queue!(stdout, cursor::Show)?;
        }
        stdout.flush()?;

        if pressed_enter {
            Ok(self.prompt.finish_async().await)
        } else {
            Ok(self.prompt.finish_default())
        }
    }

    /// Run the ui on the given writer. It will return when the user presses `Enter` or `Escape`
    /// based on the [`AsyncPrompt`] implementation.
    pub async fn run<W: io::Write>(mut self, stdout: &mut W) -> crossterm::Result<P::Output> {
        let (tw, th) = terminal::size()?;
        self.terminal_h = th;
        self.terminal_w = tw;

        let prompt = self.prompt.prompt();
        let prompt_len = u16::try_from(prompt.chars().count() + 3).expect("really big prompt");

        let raw_mode = RawMode::enable()?;
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

        let mut events = event::EventStream::new();

        loop {
            match events.next().await.unwrap()? {
                event::Event::Resize(tw, th) => {
                    self.terminal_w = tw;
                    self.terminal_h = th;
                }

                event::Event::Key(e) => {
                    let key_handled = match e.code {
                        event::KeyCode::Char('c')
                            if e.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            queue!(
                                stdout,
                                cursor::MoveTo(0, self.base_row + self.prompt.height() as u16)
                            )?;
                            drop(raw_mode);
                            if self.hide_cursor {
                                queue!(stdout, cursor::Show)?;
                            }
                            crate::exit();

                            return Err(io::Error::new(io::ErrorKind::Other, "CTRL+C").into());
                        }
                        event::KeyCode::Null => {
                            queue!(
                                stdout,
                                cursor::MoveTo(0, self.base_row + self.prompt.height() as u16)
                            )?;
                            drop(raw_mode);
                            if self.hide_cursor {
                                queue!(stdout, cursor::Show)?;
                            }
                            crate::exit();
                            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF").into());
                        }
                        event::KeyCode::Esc if self.prompt.has_default() => {
                            return self.finish(false, prompt_len, stdout).await;
                        }

                        event::KeyCode::Enter => {
                            let result = match self.prompt.try_validate_sync() {
                                Some(res) => res,
                                None => self.prompt.validate_async().await,
                            };

                            match result {
                                Ok(Validation::Finish) => {
                                    return self.finish(true, prompt_len, stdout).await;
                                }
                                Ok(Validation::Continue) => true,
                                Err(e) => {
                                    let height = self.prompt.height() + 1;
                                    self.base_row = self.adjust_scrollback(height, stdout)?;

                                    queue!(
                                        stdout,
                                        cursor::MoveTo(0, self.base_row + height as u16)
                                    )?;
                                    write!(stdout, "{} {}", ">>".dark_red(), e)?;

                                    self.set_cursor_pos(stdout)?;

                                    continue;
                                }
                            }
                        }
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

impl<P> AsyncInput<P> {
    /// Creates a new AsyncInput
    pub fn new(prompt: P) -> Self {
        Self {
            prompt,
            base_row: 0,
            base_col: 0,
            terminal_h: 0,
            terminal_w: 0,
            hide_cursor: false,
        }
    }

    /// Hides the cursor while running the input
    pub fn hide_cursor(mut self) -> Self {
        self.hide_cursor = true;
        self
    }
}

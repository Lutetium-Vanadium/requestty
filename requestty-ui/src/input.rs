use std::{
    io,
    ops::{Deref, DerefMut},
};

use super::Widget;
use crate::{
    backend::{Backend, ClearType, MoveDirection, Size},
    error,
    events::{EventIterator, KeyCode, KeyModifiers},
    layout::Layout,
    style::Stylize,
};

/// The state of a prompt on validation.
///
/// See [`Prompt::validate`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validation {
    /// If the prompt is ready to finish.
    Finish,
    /// If the state is valid, but the prompt should still persist.
    ///
    /// Unlike returning an Err, this will not show an error and is a way for the prompt to progress
    /// its internal state machine.
    Continue,
}

/// What to do after receiving `Esc`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnEsc {
    /// Stop asking the `PromptModule`. Similar effect to `Ctrl+C` except it can be distinguished in
    /// case different behaviour is needed
    Terminate,
    /// Skip the current question and move on to the next question. The question will not be asked
    /// again.
    SkipQuestion,
    /// Pressing `Esc` will not do anything, and will be ignored. This is the default behaviour.
    Ignore,
}

/// This trait should be implemented by all 'root' widgets.
///
/// It provides the functionality required only by the main controlling widget. For the trait
/// required for general rendering to terminal, see [`Widget`].
pub trait Prompt: Widget {
    /// The error type returned by validate. It can be any widget and the [render cycle] is guaranteed
    /// to be called only once.
    ///
    /// [render cycle]: widgets/trait.Widget.html#render-cycle
    type ValidateErr: Widget;

    /// The output type returned by [`Input::run`]
    type Output;

    /// Determine whether the prompt state is ready to be submitted. It is called whenever the user
    /// presses the enter key.
    ///
    /// See [`Validation`]
    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        Ok(Validation::Finish)
    }
    /// The value to return from [`Input::run`]. This will only be called once validation returns
    /// [`Validation::Finish`]
    fn finish(self) -> Self::Output;
}

/// A ui runner which implements the [render cycle].
///
/// It renders and processes events with the help of a type that implements [`Prompt`].
///
/// See [`run`](Input::run) for more information
///
/// [render cycle]: widgets/trait.Widget.html#render-cycle
#[derive(Debug)]
pub struct Input<P, B: Backend> {
    prompt: P,
    on_esc: OnEsc,
    backend: TerminalState<B>,
    base_row: u16,
    size: Size,
    render_overflow: bool,
}

impl<P, B: Backend> Input<P, B> {
    #[allow(clippy::new_ret_no_self)]
    /// Creates a new `Input`. This won't do anything until it is [run](Input::run).
    pub fn new(prompt: P, backend: &mut B) -> Input<P, &mut B> {
        // The method doesn't return self directly, as its always used with a `&mut B`,
        // and this tells the compiler that it doesn't need to consume the `&mut B`, but
        // once the Input has been dropped, it can be used again
        Input {
            prompt,
            on_esc: OnEsc::Ignore,
            backend: TerminalState::new(backend, false),
            base_row: 0,
            size: Size::default(),
            render_overflow: false,
        }
    }

    /// Hides the cursor while running the input. This won't do anything until it is [run](Input::run).
    pub fn hide_cursor(mut self) -> Self {
        self.backend.hide_cursor = true;
        self
    }

    /// What to do after receiving a `Esc`.
    ///
    /// For [`OnEsc::Terminate`] - an [`Error::Aborted`](error::ErrorKind::Aborted) will be returned.
    /// For [`OnEsc::SkipQuestion`] - the currently shown prompt will be cleared, and `Ok(None)`
    /// will be returned.
    /// For [`OnEsc::Ignore`] - no special behaviour will be applied to the `Esc` key. Like other
    /// keys, the `Esc` key will be passed to the prompt to handle.
    pub fn on_esc(mut self, on_esc: OnEsc) -> Self {
        self.on_esc = on_esc;
        self
    }
}

impl<P: Prompt, B: Backend> Input<P, B> {
    fn layout(&self) -> Layout {
        Layout::new(0, self.size).with_offset(0, self.base_row)
    }

    fn update_size(&mut self) -> io::Result<()> {
        self.size = self.backend.size()?;
        if self.size.area() == 0 {
            Err(io::Error::other(format!(
                "Invalid terminal {:?}. Both width and height must be larger than 0",
                self.size
            )))
        } else {
            Ok(())
        }
    }

    fn init(&mut self) -> io::Result<()> {
        self.backend.init()?;
        self.base_row = self.backend.get_cursor_pos()?.1;
        self.render()
    }

    fn adjust_scrollback(&mut self, height: u16) -> io::Result<u16> {
        let th = self.size.height;

        let mut base_row = self.base_row;

        if self.base_row > th.saturating_sub(height) {
            let dist = self.base_row - th.saturating_sub(height);
            base_row -= dist;
            self.backend.scroll(-(dist as i16))?;
            self.backend.move_cursor(MoveDirection::Up(dist))?;
        }

        Ok(base_row)
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.backend.hide_cursor {
            let (x, y) = self.prompt.cursor_pos(self.layout());

            if self.render_overflow && y >= self.size.height - 1 {
                // If the height of the prompt exceeds the height of the terminal a cut-off message
                // is displayed at the bottom. If the cursor is positioned on this cut-off, then we
                // hide it.
                if !self.backend.cursor_hidden {
                    self.backend.cursor_hidden = true;
                    self.backend.hide_cursor()?;
                }
            } else if self.backend.cursor_hidden {
                // Otherwise, the cursor should be visible, and currently is not. So, we show it.
                self.backend.cursor_hidden = false;
                self.backend.show_cursor()?;
            }

            self.backend.move_cursor_to(x, y)?;
        }
        self.backend.flush()
    }

    fn render_cutoff_msg(&mut self) -> io::Result<()> {
        let cross = crate::symbols::current().cross;
        self.backend.set_fg(crate::style::Color::DarkGrey)?;
        write!(
            self.backend,
            "{0} the window height is too small, the prompt has been cut-off {0}",
            cross
        )?;
        self.backend.set_fg(crate::style::Color::Reset)
    }

    fn render(&mut self) -> io::Result<()> {
        self.update_size()?;
        let height = self.prompt.height(&mut self.layout());
        self.base_row = self.adjust_scrollback(height)?;
        self.clear()?;

        self.prompt.render(&mut self.layout(), &mut *self.backend)?;
        self.render_overflow = height > self.size.height;

        if self.render_overflow {
            self.backend.move_cursor_to(0, self.size.height - 1)?;
            self.render_cutoff_msg()?;
        }

        self.flush()
    }

    fn clear(&mut self) -> io::Result<()> {
        self.backend.move_cursor_to(0, self.base_row)?;
        self.backend.clear(ClearType::FromCursorDown)
    }

    fn goto_last_line(&mut self, height: u16) -> io::Result<()> {
        self.base_row = self.adjust_scrollback(height + 1)?;
        self.backend.move_cursor_to(0, self.base_row + height)
    }

    fn print_error(&mut self, mut e: P::ValidateErr) -> io::Result<()> {
        self.update_size()?;
        let height = self.prompt.height(&mut self.layout());
        self.base_row = self.adjust_scrollback(height + 1)?;
        self.clear()?;
        self.prompt.render(&mut self.layout(), &mut *self.backend)?;

        self.goto_last_line(height)?;

        let mut layout = Layout::new(2, self.size).with_offset(0, self.base_row + height);
        let err_height = e.height(&mut layout.clone());
        self.base_row = self.adjust_scrollback(height + err_height)?;

        if self.render_overflow {
            self.backend
                .move_cursor_to(0, self.size.height - err_height - 1)?;
            self.backend.clear(ClearType::FromCursorDown)?;
            self.render_cutoff_msg()?;
            self.backend
                .move_cursor_to(0, self.size.height - err_height)?;
        }

        self.backend
            .write_styled(&crate::symbols::current().cross.red())?;
        self.backend.write_all(b" ")?;

        e.render(&mut layout, &mut *self.backend)?;

        self.flush()
    }

    fn exit(&mut self) -> io::Result<()> {
        self.update_size()?;
        let height = self.prompt.height(&mut self.layout());
        self.goto_last_line(height)?;
        self.backend.reset()
    }

    /// Display the prompt and process events until the user presses `Enter`.
    ///
    /// After the user presses `Enter`, [`validate`](Prompt::validate) will be called.
    pub fn run<E>(mut self, events: &mut E) -> error::Result<Option<P::Output>>
    where
        E: EventIterator,
    {
        self.init()?;

        loop {
            let e = events.next_event()?;

            let key_handled = match e.code {
                KeyCode::Char('c') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.exit()?;
                    return Err(error::ErrorKind::Interrupted);
                }
                KeyCode::Null => {
                    self.exit()?;
                    return Err(error::ErrorKind::Eof);
                }
                KeyCode::Esc if self.on_esc == OnEsc::Terminate => {
                    self.exit()?;
                    return Err(error::ErrorKind::Aborted);
                }
                KeyCode::Esc if self.on_esc == OnEsc::SkipQuestion => {
                    self.clear()?;
                    self.backend.reset()?;

                    return Ok(None);
                }
                KeyCode::Enter => match self.prompt.validate() {
                    Ok(Validation::Finish) => {
                        self.clear()?;
                        self.backend.reset()?;

                        return Ok(Some(self.prompt.finish()));
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
                self.render()?;
            }
        }
    }
}

#[derive(Debug)]
struct TerminalState<B: Backend> {
    backend: B,
    hide_cursor: bool,
    cursor_hidden: bool,
    enabled: bool,
}

impl<B: Backend> TerminalState<B> {
    fn new(backend: B, hide_cursor: bool) -> Self {
        Self {
            backend,
            enabled: false,
            hide_cursor,
            cursor_hidden: false,
        }
    }

    fn init(&mut self) -> io::Result<()> {
        self.enabled = true;
        if self.hide_cursor && !self.cursor_hidden {
            self.backend.hide_cursor()?;
            self.cursor_hidden = true;
        }
        self.backend.enable_raw_mode()
    }

    fn reset(&mut self) -> io::Result<()> {
        self.enabled = false;
        if self.cursor_hidden {
            self.backend.show_cursor()?;
            self.cursor_hidden = false;
        }
        self.backend.disable_raw_mode()
    }
}

impl<B: Backend> Drop for TerminalState<B> {
    fn drop(&mut self) {
        if self.enabled {
            let _ = self.reset();
        }
    }
}

impl<B: Backend> Deref for TerminalState<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: Backend> DerefMut for TerminalState<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend::TestBackend, events::TestEvents};

    #[derive(Debug, Default, Clone, Copy)]
    struct TestPrompt {
        height: u16,
    }

    impl Widget for TestPrompt {
        fn render<B: Backend>(&mut self, layout: &mut Layout, backend: &mut B) -> io::Result<()> {
            for i in 0..self.height(layout) {
                // Not the most efficient but this is a test, and it makes assertions easier
                backend.write_all(format!("Line {}", i).as_bytes())?;
                backend.move_cursor(MoveDirection::NextLine(1))?;
            }
            Ok(())
        }

        fn height(&mut self, layout: &mut Layout) -> u16 {
            layout.offset_y += self.height;
            self.height
        }

        fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
            layout.offset_cursor((0, self.height))
        }

        fn handle_key(&mut self, key: crate::events::KeyEvent) -> bool {
            todo!("{:?}", key)
        }
    }

    impl Prompt for TestPrompt {
        type ValidateErr = &'static str;

        type Output = ();

        fn finish(self) -> Self::Output {}
    }

    #[test]
    fn test_hide_cursor() {
        let mut backend = TestBackend::new((100, 20).into());
        let mut backend = Input::new(TestPrompt::default(), &mut backend)
            .hide_cursor()
            .backend;

        backend.init().unwrap();

        crate::assert_backend_snapshot!(*backend);
    }

    #[test]
    fn test_adjust_scrollback() {
        let prompt = TestPrompt::default();
        let size = (100, 20).into();

        let mut backend = TestBackend::new(size);
        backend.move_cursor_to(0, 14).unwrap();

        assert_eq!(
            Input {
                prompt,
                on_esc: OnEsc::Ignore,
                backend: TerminalState::new(&mut backend, false),
                base_row: 14,
                size,
                render_overflow: false,
            }
            .adjust_scrollback(3)
            .unwrap(),
            14
        );

        crate::assert_backend_snapshot!(backend);

        assert_eq!(
            Input {
                prompt,
                on_esc: OnEsc::Ignore,
                backend: TerminalState::new(&mut backend, false),
                base_row: 14,
                size,
                render_overflow: false,
            }
            .adjust_scrollback(6)
            .unwrap(),
            14
        );
        crate::assert_backend_snapshot!(backend);

        assert_eq!(
            Input {
                prompt,
                on_esc: OnEsc::Ignore,
                backend: TerminalState::new(&mut backend, false),
                base_row: 14,
                size,
                render_overflow: false,
            }
            .adjust_scrollback(10)
            .unwrap(),
            10
        );
        crate::assert_backend_snapshot!(backend);
    }

    #[test]
    fn test_render() {
        let prompt = TestPrompt { height: 5 };
        let size = (100, 20).into();
        let mut backend = TestBackend::new(size);
        backend.move_cursor_to(0, 5).unwrap();

        assert!(Input {
            prompt,
            on_esc: OnEsc::Ignore,
            backend: TerminalState::new(&mut backend, false),
            size,
            base_row: 5,
            render_overflow: false,
        }
        .render()
        .is_ok());

        crate::assert_backend_snapshot!(backend);
    }

    #[test]
    fn test_goto_last_line() {
        let size = (100, 20).into();
        let mut backend = TestBackend::new(size);
        backend.move_cursor_to(0, 15).unwrap();

        let mut input = Input {
            prompt: TestPrompt::default(),
            on_esc: OnEsc::Ignore,
            backend: TerminalState::new(&mut backend, false),
            size,
            base_row: 15,
            render_overflow: false,
        };

        assert!(input.goto_last_line(9).is_ok());
        assert_eq!(input.base_row, 10);
        drop(input);

        crate::assert_backend_snapshot!(backend);
    }

    #[test]
    fn test_print_error() {
        let error = "error text";
        let size = (100, 20).into();
        let mut backend = TestBackend::new(size);

        assert!(Input {
            prompt: TestPrompt { height: 5 },
            on_esc: OnEsc::Ignore,
            backend: TerminalState::new(&mut backend, true),
            base_row: 0,
            size,
            render_overflow: false,
        }
        .print_error(error)
        .is_ok());

        crate::assert_backend_snapshot!(backend);
    }

    #[test]
    fn test_zero_size() {
        let mut backend = TestBackend::new((20, 0).into());
        let err = Input::new(TestPrompt::default(), &mut backend)
            .run(&mut TestEvents::new([]))
            .expect_err("zero size should error");

        let err = match err {
            crate::ErrorKind::IoError(err) => err,
            err => panic!("expected io error, got {:?}", err),
        };

        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(
            format!("{}", err),
            "Invalid terminal Size { width: 20, height: 0 }. Both width and height must be larger than 0"
        );

        let mut backend = TestBackend::new((0, 20).into());
        let err = Input::new(TestPrompt::default(), &mut backend)
            .run(&mut TestEvents::new([]))
            .expect_err("zero size should error");

        let err = match err {
            crate::ErrorKind::IoError(err) => err,
            err => panic!("expected io error, got {:?}", err),
        };

        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(
            format!("{}", err),
            "Invalid terminal Size { width: 0, height: 20 }. Both width and height must be larger than 0"
        );
    }
}

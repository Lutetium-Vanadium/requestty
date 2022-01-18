use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyEvent},
    style::Stylize,
    widgets, Prompt, Validation, Widget,
};

use super::{Options, TransformByVal as Transform};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub(super) struct Confirm<'a> {
    default: Option<bool>,
    transform: Transform<'a, bool>,
}

struct ConfirmPrompt<'a> {
    prompt: widgets::Prompt<&'a str>,
    confirm: Confirm<'a>,
    input: widgets::CharInput,
}

impl Widget for ConfirmPrompt<'_> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        self.prompt.render(layout, b)?;
        self.input.render(layout, b)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.input.height(layout) - 1
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.input.handle_key(key)
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.input
            .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
    }
}

fn only_yn(c: char) -> Option<char> {
    match c {
        'y' | 'Y' | 'n' | 'N' => Some(c),
        _ => None,
    }
}

impl Prompt for ConfirmPrompt<'_> {
    type ValidateErr = &'static str;
    type Output = bool;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.input.value().is_some() || self.confirm.default.is_some() {
            Ok(Validation::Finish)
        } else {
            Err("Please enter y or n")
        }
    }

    fn finish(self) -> Self::Output {
        match self.input.value() {
            Some('y') | Some('Y') => true,
            Some('n') | Some('N') => false,
            _ => self
                .confirm
                .default
                .expect("Validation would fail if there was no answer and no default"),
        }
    }
}

impl<'a> Confirm<'a> {
    fn into_confirm_prompt(self, message: &'a str) -> ConfirmPrompt<'a> {
        let hint = match self.default {
            Some(true) => "Y/n",
            Some(false) => "y/N",
            None => "y/n",
        };

        ConfirmPrompt {
            prompt: widgets::Prompt::new(message).with_hint(hint),
            confirm: self,
            input: widgets::CharInput::with_filter_map(only_yn),
        }
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        on_esc: ui::OnEsc,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Option<Answer>> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_confirm_prompt(&message), b)
            .on_esc(on_esc)
            .run(events)?;

        crate::write_final!(transform, message, ans, answers, b, |ans| {
            let ans = if ans { "Yes" } else { "No" };
            b.write_styled(&ans.cyan())?;
        })
    }
}

/// The builder for a [`confirm`] prompt.
///
/// <img
///   src="https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/confirm.gif"
///   style="max-height: 11rem"
/// />
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use requestty::Question;
///
/// let confirm = Question::confirm("anonymous")
///     .message("Do you want to remain anonymous?")
///     .build();
/// ```
///
/// [`confirm`]: crate::question::Question::confirm
#[derive(Debug)]
pub struct ConfirmBuilder<'a> {
    opts: Options<'a>,
    confirm: Confirm<'a>,
}

impl<'a> ConfirmBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        ConfirmBuilder {
            opts: Options::new(name),
            confirm: Default::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .message("Do you want to remain anonymous?")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("auth") {
    ///         Some(ans) => !ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```

    on_esc
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, OnEsc};
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .on_esc(OnEsc::Terminate)
    ///     .build();
    /// ```
    }

    /// Set a default value for the confirm
    ///
    /// If the input text is empty, the `default` is taken as the answer.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .default(false)
    ///     .build();
    /// ```
    pub fn default(mut self, default: bool) -> Self {
        self.confirm.default = Some(default);
        self
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .transform(|anonymous, previous_answers, backend| {
    ///         if anonymous  {
    ///             write!(backend, "Ok, you are now anonymous!")
    ///         } else {
    ///             write!(backend, "Please enter your details in the later prompts!")
    ///         }
    ///     })
    ///     .build();
    /// ```
    by val bool; confirm
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Confirm(self.confirm))
    }
}

impl<'a> From<ConfirmBuilder<'a>> for super::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: ConfirmBuilder<'a>) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui::{backend::TestBackend, events::KeyCode, layout::Layout};

    fn confirm(default: Option<bool>, message: &str) -> ConfirmPrompt<'_> {
        Confirm {
            default,
            ..Default::default()
        }
        .into_confirm_prompt(message)
    }

    #[test]
    fn test_render() {
        let mut confirms = [
            confirm(None, "message"),
            confirm(Some(true), "message"),
            confirm(Some(false), "message"),
        ];

        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);

        let mut backend = TestBackend::new_with_layout(size, base_layout);

        for confirm in confirms.iter_mut() {
            let offsets = [21, 22, 22];
            let keys = [
                KeyEvent::from(KeyCode::Char('y')),
                KeyCode::Char('n').into(),
                KeyCode::Backspace.into(),
            ];

            let base_name = match confirm.confirm.default {
                Some(true) => "default_y",
                Some(false) => "default_n",
                None => "no_default",
            };

            for (i, (&line_offset, &key)) in offsets.iter().zip(keys.iter()).enumerate() {
                let mut layout = base_layout;
                assert!(confirm.render(&mut layout, &mut backend).is_ok());
                ui::assert_backend_snapshot!(format!("{}-{}", base_name, i), backend);
                assert_eq!(layout, base_layout.with_line_offset(line_offset));
                backend.reset_with_layout(base_layout);
                confirm.handle_key(key);
            }

            let mut layout = base_layout;
            assert!(confirm.render(&mut layout, &mut backend).is_ok());
            ui::assert_backend_snapshot!(format!("{}-{}", base_name, keys.len()), backend);
            assert_eq!(layout, base_layout.with_line_offset(21));
            backend.reset_with_layout(base_layout);
        }
    }

    #[test]
    fn test_height() {
        let mut confirms = [
            confirm(None, "message"),
            confirm(Some(true), "message"),
            confirm(Some(false), "message"),
        ];

        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);

        for confirm in confirms.iter_mut() {
            let offsets = [21, 22, 22];
            let keys = [
                KeyEvent::from(KeyCode::Char('y')),
                KeyCode::Char('n').into(),
                KeyCode::Backspace.into(),
            ];

            for (&line_offset, &key) in offsets.iter().zip(keys.iter()) {
                let mut layout = base_layout;
                assert_eq!(confirm.height(&mut layout), 1);
                assert_eq!(layout, base_layout.with_line_offset(line_offset));
                confirm.handle_key(key);
            }

            let mut layout = base_layout;
            assert_eq!(confirm.height(&mut layout), 1);
            assert_eq!(layout, base_layout.with_line_offset(21));
        }
    }

    #[test]
    fn test_cursor_pos() {
        let size = (50, 20).into();
        let layout = Layout::new(5, size);

        let message = "-".repeat(size.width as usize) + "message";

        let mut confirms = [
            (confirm(None, "message"), 0),
            (confirm(Some(true), "message"), 0),
            (confirm(Some(false), "message"), 0),
            (confirm(None, &message), 1),
            (confirm(Some(true), &message), 1),
            (confirm(Some(false), &message), 1),
        ];

        for (confirm, offset_y) in confirms.iter_mut() {
            let offset_y = *offset_y;

            let offsets = [21, 22, 22];
            let keys = [
                KeyEvent::from(KeyCode::Char('y')),
                KeyCode::Char('n').into(),
                KeyCode::Backspace.into(),
            ];

            for (&line_offset, &key) in offsets.iter().zip(keys.iter()) {
                assert_eq!(confirm.cursor_pos(layout), (line_offset, offset_y));
                confirm.handle_key(key);
            }

            assert_eq!(confirm.cursor_pos(layout), (21, offset_y));
        }
    }
}

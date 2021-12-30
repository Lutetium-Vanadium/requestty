use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyEvent},
    style::Stylize,
    widgets, Validation, Widget,
};

use super::{Filter, Options, Transform, Validate, ValidateOnKey};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub(super) struct Password<'a> {
    mask: Option<char>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    validate_on_key: ValidateOnKey<'a, str>,
    transform: Transform<'a, str>,
}

struct PasswordPrompt<'a, 'p> {
    prompt: widgets::Prompt<&'a str>,
    password: Password<'p>,
    input: widgets::StringInput,
    is_valid: bool,
    answers: &'a Answers,
}

impl ui::Prompt for PasswordPrompt<'_, '_> {
    type ValidateErr = widgets::Text<String>;
    type Output = String;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Sync(ref mut validate) = self.password.validate {
            validate(self.input.value(), self.answers)?;
        }

        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        let mut ans = self.input.finish().unwrap_or_else(String::new);

        if let Filter::Sync(filter) = self.password.filter {
            ans = filter(ans, self.answers)
        }

        ans
    }
}

impl Widget for PasswordPrompt<'_, '_> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        self.prompt.render(layout, b)?;

        // if the current input does not satisfy the on key validation, then we show its wrong by
        // using the red colour
        if !self.is_valid {
            b.set_fg(ui::style::Color::Red)?;
        }
        self.input.render(layout, b)?;
        if !self.is_valid {
            b.set_fg(ui::style::Color::Reset)?;
        }

        Ok(())
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.input.height(layout) - 1
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let handled = self.input.handle_key(key);

        match self.password.validate_on_key {
            ValidateOnKey::Sync(ref mut validate) if handled => {
                self.is_valid = validate(self.input.value(), self.answers);
            }
            _ => {}
        }

        handled
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.input
            .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
    }
}

impl<'p> Password<'p> {
    fn into_prompt<'a>(self, message: &'a str, answers: &'a Answers) -> PasswordPrompt<'a, 'p> {
        PasswordPrompt {
            prompt: widgets::Prompt::new(message)
                .with_delim(widgets::Delimiter::SquareBracket)
                .with_optional_hint(if self.mask.is_none() {
                    Some("input is hidden")
                } else {
                    None
                }),
            input: widgets::StringInput::default().password(self.mask),
            is_valid: true,
            password: self,
            answers,
        }
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_prompt(&message, answers), b).run(events)?;

        crate::write_final!(
            transform,
            message,
            &ans,
            answers,
            b,
            b.write_styled(&"[hidden]".dark_grey())?
        );

        Ok(Answer::String(ans))
    }
}

/// The builder for an [`password`] prompt.
///
/// How it looks if you set a mask:
///
/// <img
///   src="https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/password-mask.gif"
///   style="max-height: 11rem"
/// />
///
/// How it looks if you do not set a mask:
///
/// <img
///   src="https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/password-hidden.gif"
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
/// let password = Question::password("password")
///     .message("What is your password?")
///     .mask('*')
///     .build();
/// ```
///
/// [`password`]: crate::question::Question::password
#[derive(Debug)]
pub struct PasswordBuilder<'a> {
    opts: Options<'a>,
    password: Password<'a>,
}

impl<'a> PasswordBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        PasswordBuilder {
            opts: Options::new(name),
            password: Default::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let password = Question::password("password")
    ///     .message("What is your password?")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{Answers, Question};
    ///
    /// let password = Question::password("password")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("anonymous") {
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
    /// let password = Question::password("password")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Set a mask to print instead of the characters
    ///
    /// Each character when printed to the terminal will be replaced by the given mask. If a mask is
    /// not provided, then input will be hidden.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let password = Question::password("password")
    ///     .mask('*')
    ///     .build();
    /// ```
    pub fn mask(mut self, mask: char) -> Self {
        self.password.mask = Some(mask);
        self
    }

    crate::impl_filter_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// # fn encrypt(s: String) -> String { s }
    /// let password = Question::password("password")
    ///     .filter(|password, previous_answers| encrypt(password))
    ///     .build();
    /// ```
    String; password
    }

    crate::impl_validate_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let password = Question::password("password")
    ///     .validate(|password, previous_answers| if password.chars().count() >= 5 {
    ///         Ok(())
    ///     } else {
    ///         Err("Your password must be at least 5 characters long".to_owned())
    ///     })
    ///     .build();
    /// ```
    str; password
    }

    crate::impl_validate_on_key_builder! {
    /// Note, this method will have no visual effect if the password is hidden instead of masked.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let password = Question::password("password")
    ///     .validate_on_key(|password, previous_answers| password.chars().count() >= 5)
    ///     // Still required as this is the final validation and validate_on_key is purely cosmetic
    ///     .validate(|password, previous_answers| if password.chars().count() >= 5 {
    ///         Ok(())
    ///     } else {
    ///         Err("Your password must be at least 5 characters long".to_owned())
    ///     })
    ///     .build();
    /// ```
    str; password
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    /// use requestty::prompt::style::Color;
    ///
    /// let password = Question::password("password")
    ///     .transform(|password, previous_answers, backend| {
    ///         backend.set_fg(Color::Cyan)?;
    ///         for _ in password.chars() {
    ///             write!(backend, "*")?;
    ///         }
    ///         backend.set_fg(Color::Reset)
    ///     })
    ///     .build();
    /// ```
    str; password
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Password(self.password))
    }
}

impl<'a> From<PasswordBuilder<'a>> for super::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: PasswordBuilder<'a>) -> Self {
        builder.build()
    }
}
#[cfg(test)]
mod tests {
    use ui::{backend::TestBackend, layout::Layout};

    use super::*;

    #[test]
    fn test_render() {
        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);
        let answers = Answers::default();

        let masks = [(None, 33), (Some('*'), 17)];

        let mut backend = TestBackend::new_with_layout(size, base_layout);

        for &(mask, line_offset) in masks.iter() {
            let mut prompt = Password {
                mask,
                ..Default::default()
            }
            .into_prompt("message", &answers);

            let base_name = mask.map(|_| "mask").unwrap_or("no_mask");

            let mut layout = base_layout;
            backend.reset_with_layout(layout);
            assert!(prompt.render(&mut layout, &mut backend).is_ok());
            assert_eq!(layout, base_layout.with_line_offset(line_offset));
            ui::assert_backend_snapshot!(format!("{}-1", base_name), backend);

            prompt.input.set_value("3".repeat(50));

            layout = base_layout;
            backend.reset_with_layout(layout);
            assert!(prompt.render(&mut layout, &mut backend).is_ok());
            assert_eq!(
                layout,
                base_layout
                    .with_offset(0, mask.is_some() as u16)
                    .with_line_offset(line_offset)
            );
            ui::assert_backend_snapshot!(format!("{}-2", base_name), backend);
        }
    }

    #[test]
    fn test_height() {
        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);
        let answers = Answers::default();

        let masks = [(None, 33), (Some('*'), 17)];

        for &(mask, line_offset) in masks.iter() {
            let mut prompt = Password {
                mask,
                ..Default::default()
            }
            .into_prompt("message", &answers);

            let mut layout = base_layout;

            assert_eq!(prompt.height(&mut layout), 1);
            assert_eq!(layout, base_layout.with_line_offset(line_offset));
            layout = base_layout;

            prompt.input.set_value("3".repeat(50));
            assert_eq!(prompt.height(&mut layout), 1 + mask.is_some() as u16);
            assert_eq!(
                layout,
                base_layout
                    .with_offset(0, mask.is_some() as u16)
                    .with_line_offset(line_offset)
            );
        }
    }

    #[test]
    fn test_cursor_pos() {
        let size = (50, 20).into();
        let layout = Layout::new(5, size);
        let answers = Answers::default();

        let masks = [(None, 33), (Some('*'), 17)];

        for &(mask, line_offset) in masks.iter() {
            let mut prompt = Password {
                mask,
                ..Default::default()
            }
            .into_prompt("message", &answers);

            assert_eq!(prompt.cursor_pos(layout), (line_offset, 0));

            prompt.input.set_value("3".repeat(50));
            prompt.input.set_at(50);
            assert_eq!(
                prompt.cursor_pos(layout),
                (line_offset, mask.is_some() as u16)
            );
        }
    }
}

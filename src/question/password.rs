use ui::{
    backend::{Backend, Stylize},
    error,
    events::KeyEvent,
    widgets, Validation, Widget,
};

use super::{Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Password<'a> {
    mask: Option<char>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    transform: Transform<'a, str>,
}

struct PasswordPrompt<'a, 'p> {
    message: String,
    password: Password<'p>,
    input: widgets::StringInput,
    answers: &'a Answers,
}

impl ui::Prompt for PasswordPrompt<'_, '_> {
    type ValidateErr = String;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        if self.password.mask.is_none() {
            Some("[input is hidden]")
        } else {
            None
        }
    }

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

    fn has_default(&self) -> bool {
        false
    }
}

impl Widget for PasswordPrompt<'_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: ui::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.input.render(layout, b)
    }

    fn height(&mut self, layout: ui::Layout) -> u16 {
        self.input.height(layout)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.input.handle_key(key)
    }

    fn cursor_pos(&mut self, layout: ui::Layout) -> (u16, u16) {
        self.input.cursor_pos(layout)
    }
}

impl Password<'_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(
            PasswordPrompt {
                message,
                input: widgets::StringInput::default().password(self.mask),
                password: self,
                answers,
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(&"[hidden]".dark_grey())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::String(ans))
    }
}

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

    pub fn mask(mut self, mask: char) -> Self {
        self.password.mask = Some(mask);
        self
    }

    crate::impl_options_builder!();
    crate::impl_filter_builder!(String; password);
    crate::impl_validate_builder!(str; password);
    crate::impl_transform_builder!(str; password);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Password(self.password))
    }
}

impl<'a> From<PasswordBuilder<'a>> for super::Question<'a> {
    fn from(builder: PasswordBuilder<'a>) -> Self {
        builder.build()
    }
}

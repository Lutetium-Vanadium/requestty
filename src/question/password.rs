use ui::{
    backend::{Backend, Stylize},
    error,
    events::KeyEvent,
    widgets, Validation, Widget,
};

use super::{Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Password<'f, 'v, 't> {
    mask: Option<char>,
    filter: Filter<'f, String>,
    validate: Validate<'v, str>,
    transform: Transform<'t, str>,
}

struct PasswordPrompt<'f, 'v, 't, 'a> {
    message: String,
    password: Password<'f, 'v, 't>,
    input: widgets::StringInput,
    answers: &'a Answers,
}

impl ui::Prompt for PasswordPrompt<'_, '_, '_, '_> {
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
        if let Validate::Sync(ref validate) = self.password.validate {
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

impl Widget for PasswordPrompt<'_, '_, '_, '_> {
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

impl Password<'_, '_, '_> {
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
                b.write_styled("[hidden]".dark_grey())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::String(ans))
    }
}

pub struct PasswordBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    password: Password<'f, 'v, 't>,
}

impl<'m, 'w, 'f, 'v, 't> PasswordBuilder<'m, 'w, 'f, 'v, 't> {
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

    pub fn build(self) -> super::Question<'m, 'w, 'f, 'v, 't> {
        super::Question::new(self.opts, super::QuestionKind::Password(self.password))
    }
}

impl<'m, 'w, 'f, 'v, 't> From<PasswordBuilder<'m, 'w, 'f, 'v, 't>>
    for super::Question<'m, 'w, 'f, 'v, 't>
{
    fn from(builder: PasswordBuilder<'m, 'w, 'f, 'v, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(PasswordBuilder<'f, 'v, 't>; (this, opts) => {
    PasswordBuilder {
        opts,
        password: this.password
    }
});

crate::impl_filter_builder!(PasswordBuilder<'m, 'w, f, 'v, 't> String; (this, filter) => {
    PasswordBuilder {
        opts: this.opts,
        password: Password {
            filter,
            mask: this.password.mask,
            validate: this.password.validate,
            transform: this.password.transform,
        }
    }
});
crate::impl_validate_builder!(PasswordBuilder<'m, 'w, 'f, v, 't> str; (this, validate) => {
    PasswordBuilder {
        opts: this.opts,
        password: Password {
            validate,
            mask: this.password.mask,
            filter: this.password.filter,
            transform: this.password.transform,
        }
    }
});
crate::impl_transform_builder!(PasswordBuilder<'m, 'w, 'f, 'v, t> str; (this, transform) => {
    PasswordBuilder {
        opts: this.opts,
        password: Password {
            transform,
            validate: this.password.validate,
            mask: this.password.mask,
            filter: this.password.filter,
        }
    }
});

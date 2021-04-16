use std::fmt;

use crossterm::style::Colorize;
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::{none, some, Filter, Options, Transformer, Validate};

#[derive(Default)]
pub struct Password<'f, 'v, 't> {
    mask: Option<char>,
    filter: Option<Box<Filter<'f, String>>>,
    validate: Option<Box<Validate<'v, str>>>,
    transformer: Option<Box<Transformer<'t, str>>>,
}

impl fmt::Debug for Password<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Password")
            .field("mask", &self.mask)
            .field("filter", &self.filter.as_ref().map_or_else(none, some))
            .field("validate", &self.validate.as_ref().map_or_else(none, some))
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
}

struct PasswordPrompt<'f, 'v, 't> {
    message: String,
    password: Password<'f, 'v, 't>,
    input: widgets::StringInput,
}

impl ui::Prompt for PasswordPrompt<'_, '_, '_> {
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
        if let Some(ref validate) = self.password.validate {
            validate(self.input.value())?;
        }

        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        let mut ans = self.input.finish().unwrap_or_else(String::new);

        if let Some(filter) = self.password.filter {
            ans = filter(ans)
        }

        ans
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Widget for PasswordPrompt<'_, '_, '_> {
    fn render<W: std::io::Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        self.input.render(max_width, w)
    }

    fn height(&self) -> usize {
        self.input.height()
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.input.handle_key(key)
    }

    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        self.input.cursor_pos(prompt_len)
    }
}

impl Password<'_, '_, '_> {
    pub fn ask<W: std::io::Write>(mut self, message: String, w: &mut W) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let ans = ui::Input::new(PasswordPrompt {
            message,
            input: widgets::StringInput::new(widgets::no_filter as _).password(self.mask),
            password: self,
        })
        .run(w)?;

        match transformer {
            Some(transformer) => transformer(&ans, w)?,
            None => writeln!(w, "{}", "[hidden]".dark_grey())?,
        }

        Ok(Answer::String(ans))
    }
}

pub struct PasswordBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    password: Password<'f, 'v, 't>,
}

impl<'m, 'w, 'f, 'v, 't> PasswordBuilder<'m, 'w, 'f, 'v, 't> {
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
            transformer: this.password.transformer,
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
            transformer: this.password.transformer,
        }
    }
});
crate::impl_transformer_builder!(PasswordBuilder<'m, 'w, 'f, 'v, t> str; (this, transformer) => {
    PasswordBuilder {
        opts: this.opts,
        password: Password {
            transformer,
            validate: this.password.validate,
            mask: this.password.mask,
            filter: this.password.filter,
        }
    }
});

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn password<N: Into<String>>(
        name: N,
    ) -> PasswordBuilder<'static, 'static, 'static, 'static, 'static> {
        PasswordBuilder {
            opts: Options::new(name.into()),
            password: Default::default(),
        }
    }
}

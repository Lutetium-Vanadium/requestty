use crossterm::style::Colorize;
use ui::{widgets, Validation, Widget};

use crate::{error, Answer, Answers};

use super::{Filter, Options, Transformer, Validate};

#[derive(Debug, Default)]
pub struct Password<'f, 'v, 't> {
    mask: Option<char>,
    filter: Filter<'f, String>,
    validate: Validate<'v, str>,
    transformer: Transformer<'t, str>,
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

crate::cfg_async! {
#[async_trait::async_trait]
impl ui::AsyncPrompt for PasswordPrompt<'_, '_, '_, '_> {
    async fn finish_async(self) -> Self::Output {
        let ans = self.input.finish().unwrap_or_else(String::new);

        match self.password.filter {
            Filter::Async(filter) => filter(ans, self.answers).await,
            Filter::Sync(filter) => filter(ans, self.answers),
            Filter::None => ans,
        }
    }

    fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
        match self.password.validate {
            Validate::Sync(ref validate) => {
                Some(validate(self.input.value(), self.answers).map(|_| Validation::Finish))
            }
            _ => None,
        }
    }

    async fn validate_async(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Async(ref validate) = self.password.validate {
            validate(self.input.value(), self.answers).await?;
        }

        Ok(Validation::Finish)
    }
}
}

impl Widget for PasswordPrompt<'_, '_, '_, '_> {
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
    pub(crate) fn ask<W: std::io::Write>(
        mut self,
        message: String,
        answers: &Answers,
        w: &mut W,
    ) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let ans = ui::Input::new(PasswordPrompt {
            message,
            input: widgets::StringInput::default().password(self.mask),
            password: self,
            answers,
        })
        .run(w)?;

        match transformer {
            Transformer::Sync(transformer) => transformer(&ans, answers, w)?,
            _ => writeln!(w, "{}", "[hidden]".dark_grey())?,
        }

        Ok(Answer::String(ans))
    }

    crate::cfg_async! {
    pub(crate) async fn ask_async<W: std::io::Write>(
        mut self,
        message: String,
        answers: &Answers,
        w: &mut W,
    ) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let ans = ui::AsyncInput::new(PasswordPrompt {
            message,
            input: widgets::StringInput::default().password(self.mask),
            password: self,
            answers,
        })
        .run(w)
        .await?;

        match transformer {
            Transformer::Async(transformer) => transformer(&ans, answers, w).await?,
            Transformer::Sync(transformer) => transformer(&ans, answers, w)?,
            _ => writeln!(w, "{}", "[hidden]".dark_grey())?,
        }

        Ok(Answer::String(ans))
    }
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

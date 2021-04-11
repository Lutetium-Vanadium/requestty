use crossterm::style::Colorize;
use ui::{widgets, Widget};

use crate::{error, Answer};

use super::Options;

#[derive(Debug, Default)]
pub struct Password {
    pub(crate) mask: Option<char>,
}

struct PasswordPrompt {
    message: String,
    password: Password,
    input: widgets::StringInput,
}

impl ui::Prompt for PasswordPrompt {
    type ValidateErr = &'static str;
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

    fn finish(self) -> Self::Output {
        self.input.finish().unwrap_or_else(String::new)
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Widget for PasswordPrompt {
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

impl Password {
    pub fn ask<W: std::io::Write>(self, message: String, w: &mut W) -> error::Result<Answer> {
        let ans = ui::Input::new(PasswordPrompt {
            message,
            input: widgets::StringInput::new(widgets::no_filter as _).password(self.mask),
            password: self,
        })
        .run(w)?;

        writeln!(w, "{}", "[hidden]".dark_grey())?;

        Ok(Answer::String(ans))
    }
}

pub struct PasswordBuilder<'m, 'w> {
    opts: Options<'m, 'w>,
    password: Password,
}

impl<'m, 'w> PasswordBuilder<'m, 'w> {
    pub fn mask(mut self, mask: char) -> Self {
        self.password.mask = Some(mask);
        self
    }

    pub fn build(self) -> super::Question<'m, 'w> {
        super::Question::new(self.opts, super::QuestionKind::Password(self.password))
    }
}

impl<'m, 'w> From<PasswordBuilder<'m, 'w>> for super::Question<'m, 'w> {
    fn from(builder: PasswordBuilder<'m, 'w>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(PasswordBuilder; (this, opts) => {
    PasswordBuilder {
        opts,
        password: this.password
    }
});

impl super::Question<'static, 'static> {
    pub fn password<N: Into<String>>(name: N) -> PasswordBuilder<'static, 'static> {
        PasswordBuilder {
            opts: Options::new(name.into()),
            password: Default::default(),
        }
    }
}

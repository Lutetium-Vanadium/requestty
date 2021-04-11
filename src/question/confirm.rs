use crossterm::style::Colorize;
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::Options;

#[derive(Debug, Default)]
pub struct Confirm {
    default: Option<bool>,
}

struct ConfirmPrompt {
    confirm: Confirm,
    message: String,
    input: widgets::CharInput,
}

impl Widget for ConfirmPrompt {
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

fn only_yn(c: char) -> Option<char> {
    match c {
        'y' | 'Y' | 'n' | 'N' => Some(c),
        _ => None,
    }
}

impl ui::Prompt for ConfirmPrompt {
    type ValidateErr = &'static str;
    type Output = bool;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some(match self.confirm.default {
            Some(true) => "(y/n) (default y)",
            Some(false) => "(y/n) (default n)",
            None => "(y/n)",
        })
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.input.value().is_some() || self.has_default() {
            Ok(Validation::Finish)
        } else {
            Err("Please enter (y/n)")
        }
    }

    fn finish(self) -> Self::Output {
        match self.input.finish() {
            Some('y') | Some('Y') => true,
            Some('n') | Some('N') => false,
            _ => self.confirm.default.unwrap(),
        }
    }

    fn has_default(&self) -> bool {
        self.confirm.default.is_some()
    }
    fn finish_default(self) -> Self::Output {
        self.confirm.default.unwrap()
    }
}

impl Confirm {
    pub(crate) fn ask<W: std::io::Write>(
        self,
        message: String,
        w: &mut W,
    ) -> error::Result<Answer> {
        let ans = ui::Input::new(ConfirmPrompt {
            confirm: self,
            message,
            input: widgets::CharInput::new(only_yn),
        })
        .run(w)?;

        let s = if ans { "Yes" } else { "No" };

        writeln!(w, "{}", s.dark_cyan())?;

        Ok(Answer::Bool(ans))
    }
}

pub struct ConfirmBuilder<'m, 'w> {
    opts: Options<'m, 'w>,
    confirm: Confirm,
}

impl<'m, 'w> ConfirmBuilder<'m, 'w> {
    pub fn default(mut self, default: bool) -> Self {
        self.confirm.default = Some(default);
        self
    }

    pub fn build(self) -> super::Question<'m, 'w> {
        super::Question::new(self.opts, super::QuestionKind::Confirm(self.confirm))
    }
}

impl<'m, 'w> From<ConfirmBuilder<'m, 'w>> for super::Question<'m, 'w> {
    fn from(builder: ConfirmBuilder<'m, 'w>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(ConfirmBuilder; (this, opts) => {
    ConfirmBuilder {
        opts,
        confirm: this.confirm,
    }
});

impl super::Question<'static, 'static> {
    pub fn confirm<N: Into<String>>(name: N) -> ConfirmBuilder<'static, 'static> {
        ConfirmBuilder {
            opts: Options::new(name.into()),
            confirm: Default::default(),
        }
    }
}

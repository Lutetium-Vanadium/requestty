use std::fmt;

use crossterm::style::Colorize;
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::{none, some, Options, TransformerV};

#[derive(Default)]
pub struct Confirm<'t> {
    default: Option<bool>,
    transformer: Option<Box<TransformerV<'t, bool>>>,
}

impl fmt::Debug for Confirm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Confirm")
            .field("default", &self.default)
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
}

struct ConfirmPrompt<'t> {
    confirm: Confirm<'t>,
    message: String,
    input: widgets::CharInput,
}

impl Widget for ConfirmPrompt<'_> {
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

impl ui::Prompt for ConfirmPrompt<'_> {
    type ValidateErr = &'static str;
    type Output = bool;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some(match self.confirm.default {
            Some(true) => "(Y/n)",
            Some(false) => "(y/N)",
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

impl Confirm<'_> {
    pub(crate) fn ask<W: std::io::Write>(
        mut self,
        message: String,
        w: &mut W,
    ) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let ans = ui::Input::new(ConfirmPrompt {
            confirm: self,
            message,
            input: widgets::CharInput::new(only_yn),
        })
        .run(w)?;

        match transformer {
            Some(transformer) => transformer(ans, w)?,
            None => {
                let ans = if ans { "Yes" } else { "No" };

                writeln!(w, "{}", ans.dark_cyan())?;
            }
        }

        Ok(Answer::Bool(ans))
    }
}

pub struct ConfirmBuilder<'m, 'w, 't> {
    opts: Options<'m, 'w>,
    confirm: Confirm<'t>,
}

impl<'m, 'w, 't> ConfirmBuilder<'m, 'w, 't> {
    pub fn default(mut self, default: bool) -> Self {
        self.confirm.default = Some(default);
        self
    }

    pub fn build(self) -> super::Question<'m, 'w, 'static, 'static, 't> {
        super::Question::new(self.opts, super::QuestionKind::Confirm(self.confirm))
    }
}

impl<'m, 'w, 't> From<ConfirmBuilder<'m, 'w, 't>>
    for super::Question<'m, 'w, 'static, 'static, 't>
{
    fn from(builder: ConfirmBuilder<'m, 'w, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(ConfirmBuilder<'t>; (this, opts) => {
    ConfirmBuilder {
        opts,
        confirm: this.confirm,
    }
});

crate::impl_transformer_builder!(by val ConfirmBuilder<'m, 'w, t> bool; (this, transformer) => {
    ConfirmBuilder {
        opts: this.opts,
        confirm: Confirm {
            transformer,
            default: this.confirm.default,
        }
    }
});

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn confirm<N: Into<String>>(name: N) -> ConfirmBuilder<'static, 'static, 'static> {
        ConfirmBuilder {
            opts: Options::new(name.into()),
            confirm: Default::default(),
        }
    }
}

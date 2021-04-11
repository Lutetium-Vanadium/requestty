use crossterm::style::Colorize;
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::Options;

#[derive(Debug, Default)]
pub struct Input {
    default: Option<String>,
}

struct InputPrompt {
    message: String,
    input: widgets::StringInput,
    /// The default value wrapped with brackets
    hint: Option<String>,
}

impl Widget for InputPrompt {
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

impl ui::Prompt for InputPrompt {
    type ValidateErr = &'static str;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        self.hint.as_ref().map(String::as_ref)
    }

    fn finish(self) -> Self::Output {
        let hint = self.hint;
        self.input
            .finish()
            .unwrap_or_else(|| remove_brackets(hint.unwrap()))
    }
    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.input.has_value() || self.has_default() {
            Ok(Validation::Finish)
        } else {
            Err("Please enter a string")
        }
    }
    fn has_default(&self) -> bool {
        self.hint.is_some()
    }
    fn finish_default(self) -> <Self as ui::Prompt>::Output {
        remove_brackets(self.hint.unwrap())
    }
}

impl Input {
    pub fn ask<W: std::io::Write>(mut self, message: String, w: &mut W) -> error::Result<Answer> {
        if let Some(ref mut default) = self.default {
            default.insert(0, '(');
            default.push(')');
        }

        let ans = ui::Input::new(InputPrompt {
            message,
            hint: self.default,
            input: widgets::StringInput::default(),
        })
        .run(w)?;

        writeln!(w, "{}", ans.as_str().dark_cyan())?;

        Ok(Answer::String(ans))
    }
}

pub struct InputBuilder<'m, 'w> {
    opts: Options<'m, 'w>,
    input: Input,
}

impl super::Question<'static, 'static> {
    pub fn input<N: Into<String>>(name: N) -> InputBuilder<'static, 'static> {
        InputBuilder {
            opts: Options::new(name.into()),
            input: Default::default(),
        }
    }
}

impl<'m, 'w> InputBuilder<'m, 'w> {
    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.input.default = Some(default.into());
        self
    }

    pub fn build(self) -> super::Question<'m, 'w> {
        super::Question::new(self.opts, super::QuestionKind::Input(self.input))
    }
}

impl<'m, 'w> From<InputBuilder<'m, 'w>> for super::Question<'m, 'w> {
    fn from(builder: InputBuilder<'m, 'w>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(InputBuilder; (this, opts) => {
    InputBuilder {
        opts,
        input: this.input,
    }
});

fn remove_brackets(mut s: String) -> String {
    s.remove(0);
    s.pop();
    s
}

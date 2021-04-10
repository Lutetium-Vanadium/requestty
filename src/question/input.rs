use crossterm::style::Colorize;
use ui::{widgets, Widget};

use crate::{error, Answer};

use super::Options;

pub struct Input {
    // FIXME: reference instead?
    pub(crate) default: String,
}

struct InputPrompt {
    input_opts: Input,
    opts: Options,
    input: widgets::StringInput,
    hint: String,
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
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        Some(&self.hint)
    }

    fn finish(self) -> Self::Output {
        self.input.finish().unwrap_or(self.input_opts.default)
    }
    fn finish_default(self) -> <Self as ui::Prompt>::Output {
        self.input_opts.default
    }
}

impl Input {
    pub fn ask<W: std::io::Write>(self, opts: Options, w: &mut W) -> error::Result<Answer> {
        let ans = ui::Input::new(InputPrompt {
            opts,
            hint: format!("({})", self.default),
            input_opts: self,
            input: widgets::StringInput::default(),
        })
        .run(w)?;

        writeln!(w, "{}", ans.as_str().dark_cyan())?;

        Ok(Answer::String(ans))
    }
}

impl super::Question {
    pub fn input(name: String, message: String, default: String) -> Self {
        Self::new(name, message, super::QuestionKind::Input(Input { default }))
    }
}

use crossterm::style::Colorize;
use ui::{widgets, Widget};

use crate::{error, Answer};

use super::Options;

pub struct Confirm {
    pub(crate) default: bool,
}

struct ConfirmPrompt {
    confirm: Confirm,
    opts: Options,
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
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        if self.confirm.default {
            Some("(y/n) (default y)")
        } else {
            Some("(y/n) (default n)")
        }
    }

    fn finish(self) -> Self::Output {
        match self.input.finish() {
            Some('y') | Some('Y') => true,
            Some('n') | Some('N') => true,
            _ => unreachable!(),
        }
    }

    fn finish_default(self) -> Self::Output {
        self.confirm.default
    }
}

impl Confirm {
    pub(crate) fn ask<W: std::io::Write>(
        self,
        opts: super::Options,
        w: &mut W,
    ) -> error::Result<Answer> {
        let ans = ui::Input::new(ConfirmPrompt {
            confirm: self,
            opts,
            input: widgets::CharInput::new(only_yn),
        })
        .run(w)?;

        let s = if ans { "Yes" } else { "No" };

        writeln!(w, "{}", s.dark_cyan())?;

        Ok(Answer::Bool(ans))
    }
}

impl super::Question {
    pub fn confirm(name: String, message: String, default: bool) -> Self {
        Self::new(
            name,
            message,
            super::QuestionKind::Confirm(Confirm { default }),
        )
    }
}

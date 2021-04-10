use crossterm::style::Colorize;
use ui::{widgets, Widget};

use crate::{error, Answer};

use super::Options;

pub struct Password {
    pub(crate) mask: Option<char>,
}

struct PasswordPrompt {
    password: Password,
    input: widgets::StringInput,
    opts: Options,
}

impl ui::Prompt for PasswordPrompt {
    type ValidateErr = &'static str;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.opts.message
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

    fn finish_default(self) -> Self::Output {
        unreachable!()
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
    pub fn ask<W: std::io::Write>(self, opts: Options, w: &mut W) -> error::Result<Answer> {
        let ans = ui::Input::new(PasswordPrompt {
            input: widgets::StringInput::new(widgets::no_filter as _).password(self.mask),
            password: self,
            opts,
        })
        .run(w)?;

        writeln!(w, "{}", "[hidden]".dark_grey())?;

        Ok(Answer::String(ans))
    }
}

impl super::Question {
    pub fn password(name: String, message: String) -> Self {
        Self::new(
            name,
            message,
            super::QuestionKind::Password(Password { mask: None }),
        )
    }

    pub fn with_mask(mut self, mask: char) -> Self {
        if let super::QuestionKind::Password(ref mut p) = self.kind {
            p.mask = Some(mask);
        } else {
            unreachable!("with_mask should only be called when a question is password")
        }

        self
    }
}

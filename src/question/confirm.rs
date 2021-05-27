use ui::{
    backend::{Backend, Stylize},
    error,
    events::KeyEvent,
    widgets, Prompt, Validation, Widget,
};

use super::{Options, TransformByVal as Transform};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Confirm<'a> {
    default: Option<bool>,
    transform: Transform<'a, bool>,
}

struct ConfirmPrompt<'a> {
    confirm: Confirm<'a>,
    message: String,
    input: widgets::CharInput,
}

impl Widget for ConfirmPrompt<'_> {
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

fn only_yn(c: char) -> Option<char> {
    match c {
        'y' | 'Y' | 'n' | 'N' => Some(c),
        _ => None,
    }
}

impl Prompt for ConfirmPrompt<'_> {
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
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(
            ConfirmPrompt {
                confirm: self,
                message,
                input: widgets::CharInput::new(only_yn),
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(ans, answers, b)?,
            _ => {
                let ans = if ans { "Yes" } else { "No" };
                b.write_styled(ans.cyan())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::Bool(ans))
    }
}

pub struct ConfirmBuilder<'a> {
    opts: Options<'a>,
    confirm: Confirm<'a>,
}

impl<'a> ConfirmBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        ConfirmBuilder {
            opts: Options::new(name),
            confirm: Default::default(),
        }
    }

    pub fn default(mut self, default: bool) -> Self {
        self.confirm.default = Some(default);
        self
    }

    crate::impl_options_builder!();
    crate::impl_transform_builder!(by val bool; confirm);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Confirm(self.confirm))
    }
}

impl<'a> From<ConfirmBuilder<'a>> for super::Question<'a> {
    fn from(builder: ConfirmBuilder<'a>) -> Self {
        builder.build()
    }
}

use ui::{
    backend::Backend, error, events::KeyEvent, style::Stylize, widgets, Prompt, Validation, Widget,
};

use super::{Options, TransformByVal as Transform};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Confirm<'a> {
    default: Option<bool>,
    transform: Transform<'a, bool>,
}

struct ConfirmPrompt<'a> {
    prompt: widgets::Prompt<&'a str>,
    confirm: Confirm<'a>,
    input: widgets::CharInput,
}

impl Widget for ConfirmPrompt<'_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.prompt.render(layout, b)?;
        self.input.render(layout, b)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.input.height(layout) - 1
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.input.handle_key(key)
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.input
            .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
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

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.input.value().is_some() || self.confirm.default.is_some() {
            Ok(Validation::Finish)
        } else {
            Err("Please enter y or n")
        }
    }

    fn finish(self) -> Self::Output {
        match self.input.value() {
            Some('y') | Some('Y') => true,
            Some('n') | Some('N') => false,
            _ => self.confirm.default.unwrap(),
        }
    }
}

impl<'a> Confirm<'a> {
    fn into_confirm_prompt(self, message: &'a str) -> ConfirmPrompt<'a> {
        let hint = match self.default {
            Some(true) => "Y/n",
            Some(false) => "y/N",
            None => "y/n",
        };

        ConfirmPrompt {
            prompt: widgets::Prompt::new(message).with_hint(hint),
            confirm: self,
            input: widgets::CharInput::with_filter_map(only_yn),
        }
    }

    pub(crate) fn ask<B: Backend, E: Iterator<Item = error::Result<KeyEvent>>>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_confirm_prompt(&message), b).run(events)?;

        crate::write_final!(transform, message, ans, answers, b, {
            let ans = if ans { "Yes" } else { "No" };
            b.write_styled(&ans.cyan())?;
        });

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

#[cfg(test)]
mod tests {
    use super::*;
    use ui::{backend::TestBackend, events::KeyCode, layout::Layout};

    fn confirm(default: Option<bool>, message: &str) -> ConfirmPrompt {
        Confirm {
            default,
            ..Default::default()
        }
        .into_confirm_prompt(message)
    }

    #[test]
    fn test_render() {
        let mut confirms = [
            confirm(None, "message"),
            confirm(Some(true), "message"),
            confirm(Some(false), "message"),
        ];

        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);

        let mut backend = TestBackend::new_with_layout(size, base_layout);

        for confirm in confirms.iter_mut() {
            let offsets = [21, 22, 22];
            let keys = [
                KeyEvent::from(KeyCode::Char('y')),
                KeyCode::Char('n').into(),
                KeyCode::Backspace.into(),
            ];

            let base_name = match confirm.confirm.default {
                Some(true) => "default_y",
                Some(false) => "default_n",
                None => "no_default",
            };

            for (i, (&line_offset, &key)) in offsets.iter().zip(keys.iter()).enumerate() {
                let mut layout = base_layout;
                assert!(confirm.render(&mut layout, &mut backend).is_ok());
                ui::assert_backend_snapshot!(format!("{}-{}", base_name, i), backend);
                assert_eq!(layout, base_layout.with_line_offset(line_offset));
                backend.reset_with_layout(base_layout);
                confirm.handle_key(key);
            }

            let mut layout = base_layout;
            assert!(confirm.render(&mut layout, &mut backend).is_ok());
            ui::assert_backend_snapshot!(format!("{}-{}", base_name, keys.len()), backend);
            assert_eq!(layout, base_layout.with_line_offset(21));
            backend.reset_with_layout(base_layout);
        }
    }

    #[test]
    fn test_height() {
        let mut confirms = [
            confirm(None, "message"),
            confirm(Some(true), "message"),
            confirm(Some(false), "message"),
        ];

        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);

        for confirm in confirms.iter_mut() {
            let offsets = [21, 22, 22];
            let keys = [
                KeyEvent::from(KeyCode::Char('y')),
                KeyCode::Char('n').into(),
                KeyCode::Backspace.into(),
            ];

            for (&line_offset, &key) in offsets.iter().zip(keys.iter()) {
                let mut layout = base_layout;
                assert_eq!(confirm.height(&mut layout), 1);
                assert_eq!(layout, base_layout.with_line_offset(line_offset));
                confirm.handle_key(key);
            }

            let mut layout = base_layout;
            assert_eq!(confirm.height(&mut layout), 1);
            assert_eq!(layout, base_layout.with_line_offset(21));
        }
    }

    #[test]
    fn test_cursor_pos() {
        let mut confirms = [
            confirm(None, "message"),
            confirm(Some(true), "message"),
            confirm(Some(false), "message"),
        ];

        let size = (50, 20).into();
        let layout = Layout::new(5, size);

        for confirm in confirms.iter_mut() {
            let offsets = [21, 22, 22];
            let keys = [
                KeyEvent::from(KeyCode::Char('y')),
                KeyCode::Char('n').into(),
                KeyCode::Backspace.into(),
            ];

            for (&line_offset, &key) in offsets.iter().zip(keys.iter()) {
                assert_eq!(confirm.cursor_pos(layout), (line_offset, 0));
                confirm.handle_key(key);
            }

            assert_eq!(confirm.cursor_pos(layout), (21, 0));
        }
    }
}

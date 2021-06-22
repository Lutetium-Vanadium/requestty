use ui::{backend::Backend, error, events::KeyEvent, style::Stylize, widgets, Validation, Widget};

use super::{Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Password<'a> {
    mask: Option<char>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    transform: Transform<'a, str>,
}

struct PasswordPrompt<'a, 'p> {
    prompt: widgets::Prompt<&'a str>,
    password: Password<'p>,
    input: widgets::StringInput,
    answers: &'a Answers,
}

impl ui::Prompt for PasswordPrompt<'_, '_> {
    type ValidateErr = String;
    type Output = String;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Sync(ref mut validate) = self.password.validate {
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

impl Widget for PasswordPrompt<'_, '_> {
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

impl Password<'_> {
    pub(crate) fn ask<B: Backend, E: Iterator<Item = error::Result<KeyEvent>>>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(
            PasswordPrompt {
                prompt: widgets::Prompt::new(&*message)
                    .with_delim(widgets::Delimiter::SquareBracket)
                    .with_optional_hint(if self.mask.is_none() {
                        Some("input is hidden")
                    } else {
                        None
                    }),
                input: widgets::StringInput::default().password(self.mask),
                password: self,
                answers,
            },
            b,
        )
        .run(events)?;

        crate::write_final!(
            transform,
            message,
            &ans,
            answers,
            b,
            b.write_styled(&"[hidden]".dark_grey())?
        );

        Ok(Answer::String(ans))
    }
}

pub struct PasswordBuilder<'a> {
    opts: Options<'a>,
    password: Password<'a>,
}

impl<'a> PasswordBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        PasswordBuilder {
            opts: Options::new(name),
            password: Default::default(),
        }
    }

    pub fn mask(mut self, mask: char) -> Self {
        self.password.mask = Some(mask);
        self
    }

    crate::impl_options_builder!();
    crate::impl_filter_builder!(String; password);
    crate::impl_validate_builder!(str; password);
    crate::impl_transform_builder!(str; password);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Password(self.password))
    }
}

impl<'a> From<PasswordBuilder<'a>> for super::Question<'a> {
    fn from(builder: PasswordBuilder<'a>) -> Self {
        builder.build()
    }
}

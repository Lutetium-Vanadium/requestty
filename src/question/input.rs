use ui::{
    backend::{Backend, Stylize},
    error,
    events::KeyEvent,
    widgets, Prompt, Validation, Widget,
};

use super::{Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Input<'f, 'v, 't> {
    default: Option<String>,
    filter: Filter<'f, String>,
    validate: Validate<'v, str>,
    transform: Transform<'t, str>,
}

struct InputPrompt<'f, 'v, 't, 'a> {
    message: String,
    input_opts: Input<'f, 'v, 't>,
    input: widgets::StringInput,
    answers: &'a Answers,
}

impl Widget for InputPrompt<'_, '_, '_, '_> {
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

impl Prompt for InputPrompt<'_, '_, '_, '_> {
    type ValidateErr = String;
    type Output = String;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        self.input_opts.default.as_ref().map(String::as_ref)
    }

    fn finish(self) -> Self::Output {
        let hint = self.input_opts.default;
        let mut ans = self
            .input
            .finish()
            .unwrap_or_else(|| remove_brackets(hint.unwrap()));

        if let Filter::Sync(filter) = self.input_opts.filter {
            ans = filter(ans, self.answers);
        }

        ans
    }
    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if !self.input.has_value() {
            if self.has_default() {
                return Ok(Validation::Finish);
            } else {
                return Err("Please enter a string".into());
            }
        }

        if let Validate::Sync(ref validate) = self.input_opts.validate {
            validate(self.input.value(), self.answers)?;
        }

        Ok(Validation::Finish)
    }
    fn has_default(&self) -> bool {
        self.input_opts.default.is_some()
    }
    fn finish_default(self) -> <Self as ui::Prompt>::Output {
        remove_brackets(self.input_opts.default.unwrap())
    }
}

impl Input<'_, '_, '_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
        if let Some(ref mut default) = self.default {
            default.insert(0, '(');
            default.push(')');
        }

        let transform = self.transform.take();

        let ans = ui::Input::new(
            InputPrompt {
                message,
                input_opts: self,
                input: widgets::StringInput::default(),
                answers,
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(ans.as_str().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::String(ans))
    }
}

pub struct InputBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    input: Input<'f, 'v, 't>,
}

impl<'m, 'w, 'f, 'v, 't> InputBuilder<'m, 'w, 'f, 'v, 't> {
    pub(crate) fn new(name: String) -> Self {
        InputBuilder {
            opts: Options::new(name),
            input: Default::default(),
        }
    }

    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.input.default = Some(default.into());
        self
    }

    pub fn build(self) -> super::Question<'m, 'w, 'f, 'v, 't> {
        super::Question::new(self.opts, super::QuestionKind::Input(self.input))
    }
}

crate::impl_filter_builder!(InputBuilder<'m, 'w, f, 'v, 't> String; (this, filter) => {
    InputBuilder {
        opts: this.opts,
        input: Input {
            filter,
            default: this.input.default,
            validate: this.input.validate,
            transform: this.input.transform,
        }
    }
});
crate::impl_validate_builder!(InputBuilder<'m, 'w, 'f, v, 't> str; (this, validate) => {
    InputBuilder {
        opts: this.opts,
        input: Input {
            validate,
            default: this.input.default,
            filter: this.input.filter,
            transform: this.input.transform,
        }
    }
});
crate::impl_transform_builder!(InputBuilder<'m, 'w, 'f, 'v, t> str; (this, transform) => {
    InputBuilder {
        opts: this.opts,
        input: Input {
            transform,
            validate: this.input.validate,
            default: this.input.default,
            filter: this.input.filter,
        }
    }
});

impl<'m, 'w, 'f, 'v, 't> From<InputBuilder<'m, 'w, 'f, 'v, 't>>
    for super::Question<'m, 'w, 'f, 'v, 't>
{
    fn from(builder: InputBuilder<'m, 'w, 'f, 'v, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(InputBuilder<'f, 'v, 't>; (this, opts) => {
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

use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyCode, KeyEvent},
    style::Stylize,
    widgets, Prompt, Validation, Widget,
};

use super::{AutoComplete, ChoiceList, Filter, Transform, Validate};
use crate::{Answer, Answers};

pub use builder::InputBuilder;

mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct Input<'a> {
    default: Option<String>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    transform: Transform<'a, str>,
    auto_complete: AutoComplete<'a, String>,
    page_size: usize,
    should_loop: bool,
}

impl<'a> Default for Input<'a> {
    fn default() -> Self {
        Self {
            default: None,
            filter: Filter::None,
            validate: Validate::None,
            transform: Transform::None,
            auto_complete: AutoComplete::None,
            page_size: 15,
            should_loop: true,
        }
    }
}

type CompletionSelector = widgets::Select<ChoiceList<widgets::Text<String>>>;

struct InputPrompt<'i, 'a> {
    prompt: widgets::Prompt<&'a str, String>,
    input_opts: Input<'i>,
    input: widgets::StringInput,
    /// When the select is Some, then currently the user is selecting from the
    /// auto complete options. The select must not be used directly, and instead by used
    /// through `select`. See `select_op`s documentation for more.
    select: Option<CompletionSelector>,
    answers: &'a Answers,
}

impl InputPrompt<'_, '_> {
    fn maybe_select_op<T, F: FnOnce(&mut CompletionSelector) -> T>(&mut self, op: F) -> Option<T> {
        let mut res = None;

        let Self { input, select, .. } = self;

        if let Some(select) = select {
            input.replace_with(|mut s| {
                std::mem::swap(
                    &mut s,
                    &mut select.selected_mut().as_mut().unwrap_choice().text,
                );
                res = Some(op(select));
                std::mem::swap(
                    &mut s,
                    &mut select.selected_mut().as_mut().unwrap_choice().text,
                );
                s
            });
        }

        res
    }
}

impl Widget for InputPrompt<'_, '_> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        self.prompt.render(layout, b)?;
        self.input.render(layout, b)?;
        self.maybe_select_op(|select| select.render(layout, b))
            .transpose()?;
        Ok(())
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        let mut height = self.prompt.height(layout) + self.input.height(layout) - 1;

        if let Some(picker_height) = self.maybe_select_op(|select| select.height(layout)) {
            height += picker_height - 1;
        }
        height
    }

    fn handle_key(&mut self, mut key: KeyEvent) -> bool {
        let Self {
            answers,
            input_opts,
            input,
            select,
            ..
        } = self;

        match input_opts.auto_complete {
            AutoComplete::Sync(ref mut ac) if key.code == KeyCode::Tab => {
                if select.is_some() {
                    key.code = KeyCode::Down;
                } else {
                    let page_size = input_opts.page_size;
                    let should_loop = input_opts.should_loop;

                    input.replace_with(|s| {
                        let mut completions = ac(s, answers);
                        assert!(!completions.is_empty());
                        if completions.len() == 1 {
                            completions.pop().unwrap()
                        } else {
                            let res = std::mem::take(&mut completions[0]);

                            let mut choices: ChoiceList<_> =
                                completions.into_iter().map(widgets::Text::new).collect();
                            choices.set_page_size(page_size);
                            choices.set_should_loop(should_loop);

                            *select = Some(widgets::Select::new(choices));

                            res
                        }
                    });
                    return true;
                }
            }
            _ => {}
        }

        if input.handle_key(key) {
            *select = None;
            return true;
        }

        self.maybe_select_op(|select| select.handle_key(key))
            .unwrap_or(false)
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.input
            .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
    }
}

impl Prompt for InputPrompt<'_, '_> {
    type ValidateErr = widgets::Text<String>;
    type Output = String;

    fn finish(self) -> Self::Output {
        let prompt = self.prompt;
        let mut ans = self
            .input
            .finish()
            .unwrap_or_else(|| prompt.into_hint().unwrap_or_else(String::new));

        if let Filter::Sync(filter) = self.input_opts.filter {
            ans = filter(ans, self.answers);
        }

        ans
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.select.is_some() {
            self.select = None;
            return Ok(Validation::Continue);
        }

        if !self.input.has_value() && self.prompt.hint().is_some() {
            return Ok(Validation::Finish);
        }

        if let Validate::Sync(ref mut validate) = self.input_opts.validate {
            validate(self.input.value(), self.answers)?;
        }

        Ok(Validation::Finish)
    }
}

impl<'i> Input<'i> {
    fn into_input_prompt<'a>(
        mut self,
        message: &'a str,
        answers: &'a Answers,
    ) -> InputPrompt<'i, 'a> {
        InputPrompt {
            prompt: widgets::Prompt::new(message).with_optional_hint(self.default.take()),
            input_opts: self,
            input: widgets::StringInput::default(),
            select: None,
            answers,
        }
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_input_prompt(&message, answers), b).run(events)?;

        crate::write_final!(
            transform,
            message,
            &ans,
            answers,
            b,
            b.write_styled(&ans.as_str().cyan())?
        );

        Ok(Answer::String(ans))
    }
}

use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyCode, KeyEvent},
    style::Stylize,
    widgets, Prompt, Validation, Widget,
};

use super::{AutoComplete, ChoiceList, Filter, Transform, Validate, ValidateOnKey};
use crate::{Answer, Answers};

pub use builder::InputBuilder;

mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(super) struct Input<'a> {
    default: Option<(String, usize)>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    validate_on_key: ValidateOnKey<'a, str>,
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
            validate_on_key: ValidateOnKey::None,
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
    is_valid: bool,
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

    /// Returns the remaining default text if the current input is a substring of it
    fn get_remaining_default(&self) -> Option<&str> {
        if self.select.is_none() {
            if let Some((ref default, _)) = self.input_opts.default {
                let input = self.input.value();
                if default.starts_with(self.input.value()) {
                    return Some(&default[input.len()..]);
                }
            }
        }

        None
    }
}

impl Widget for InputPrompt<'_, '_> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        let mut original_layout = *layout;
        self.prompt.render(layout, b)?;

        // if the current input does not satisfy the on key validation, then we show its wrong by
        // using the red colour
        if !self.is_valid {
            b.set_fg(ui::style::Color::Red)?;
        }
        self.input.render(layout, b)?;
        if !self.is_valid {
            b.set_fg(ui::style::Color::Reset)?;
        }

        if let Some(default) = self.get_remaining_default() {
            b.set_fg(ui::style::Color::DarkGrey)?;
            write!(b, "{}", default)?;
            b.set_fg(ui::style::Color::Reset)?;
            // We need to update the layout to reflect the rest of the hint that is rendered.
            // Instead of doing the math to compute where the cursor ends after rendering, we use
            // the height function which already calculates it.
            self.height(&mut original_layout);
            *layout = original_layout;
        } else {
            self.maybe_select_op(|select| select.render(layout, b))
                .transpose()?;
        }

        Ok(())
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        let mut height = self.prompt.height(layout) - 1;

        if self.get_remaining_default().is_some() {
            let mut width = self.input_opts.default.as_ref().unwrap().1 as u16;

            if width > layout.line_width() {
                width -= layout.line_width();

                layout.line_offset = width % layout.width;
                layout.offset_y += 1 + width / layout.width;

                height += 2 + width / layout.width;
            } else {
                layout.line_offset += width;
                height += 1;
            }
        } else {
            height = self.input.height(layout);
        }

        if let Some(picker_height) = self.maybe_select_op(|select| select.height(layout)) {
            height += picker_height - 1;
        }
        height
    }

    fn handle_key(&mut self, mut key: KeyEvent) -> bool {
        if key.code == KeyCode::Tab {
            if let AutoComplete::Sync(ref mut ac) = self.input_opts.auto_complete {
                if self.select.is_some() {
                    key.code = KeyCode::Down;
                } else {
                    let page_size = self.input_opts.page_size;
                    let should_loop = self.input_opts.should_loop;

                    let Self {
                        input,
                        answers,
                        select,
                        ..
                    } = self;

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
            } else if self.get_remaining_default().is_some() {
                let (default, default_len) = self.input_opts.default.as_ref().unwrap();
                self.input.set_value(default.clone());
                self.input.set_at(*default_len);
                self.is_valid = true;
                return true;
            }
        }

        if self.input.handle_key(key) {
            if let ValidateOnKey::Sync(ref mut validate) = self.input_opts.validate_on_key {
                self.is_valid = validate(self.input.value(), self.answers);
            }

            self.select = None;
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
    fn into_input_prompt<'a>(self, message: &'a str, answers: &'a Answers) -> InputPrompt<'i, 'a> {
        InputPrompt {
            prompt: widgets::Prompt::new(message),
            input_opts: self,
            input: widgets::StringInput::default(),
            select: None,
            is_valid: true,
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

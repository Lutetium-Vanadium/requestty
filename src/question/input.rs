use ui::{
    backend::{Backend, Stylize},
    error,
    events::{KeyCode, KeyEvent},
    widgets, Prompt, Validation, Widget,
};

use super::{AutoComplete, ChoiceList, Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Input<'a> {
    default: Option<String>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    transform: Transform<'a, str>,
    auto_complete: AutoComplete<'a, String>,
}

struct InputPrompt<'i, 'a> {
    message: String,
    input_opts: Input<'i>,
    input: widgets::StringInput,
    /// When the picker is Some, then currently the user is selecting from the
    /// auto complete options. The picker must not be used directly, and instead by used
    /// through `picker_op`. See `picker_op`s documentation for more.
    picker: Option<widgets::ListPicker<ChoiceList<String>>>,
    answers: &'a Answers,
}

#[inline]
/// Calls a function with the given picker. Anytime the picker is used, it must be used
/// through this function. This is is because the selected element of the picker doesn't
/// actually contain the element, it is contained by the input. This function
/// temporarily swaps the picker's selected item and the input, performs the function,
/// and swaps back.
fn picker_op<T, F: FnOnce(&mut widgets::ListPicker<ChoiceList<String>>) -> T>(
    input: &mut widgets::StringInput,
    picker: &mut widgets::ListPicker<ChoiceList<String>>,
    op: F,
) -> T {
    let mut res = None;

    input.replace_with(|mut s| {
        std::mem::swap(&mut s, picker.selected_mut().as_mut().unwrap_choice());
        res = Some(op(picker));
        std::mem::swap(&mut s, picker.selected_mut().as_mut().unwrap_choice());
        s
    });

    res.unwrap()
}

impl Widget for InputPrompt<'_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: ui::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.input.render(layout, b)?;
        if let Some(ref mut picker) = self.picker {
            picker_op(&mut self.input, picker, |picker| picker.render(layout, b))?;
        }
        Ok(())
    }

    fn height(&mut self, layout: ui::Layout) -> u16 {
        let mut height = self.input.height(layout);
        if let Some(ref mut picker) = self.picker {
            height +=
                picker_op(&mut self.input, picker, |picker| picker.height(layout))
                    - 1;
        }
        height
    }

    fn handle_key(&mut self, mut key: KeyEvent) -> bool {
        let Self {
            answers,
            input_opts,
            input,
            picker,
            ..
        } = self;

        match input_opts.auto_complete {
            AutoComplete::Sync(ref mut ac) if key.code == KeyCode::Tab => {
                if picker.is_some() {
                    key.code = KeyCode::Down;
                } else {
                    input.replace_with(|s| {
                        let mut completions = ac(s, answers);
                        assert!(!completions.is_empty());
                        if completions.len() == 1 {
                            completions.pop().unwrap()
                        } else {
                            let res = std::mem::take(&mut completions[0]);

                            *picker = Some(widgets::ListPicker::new(
                                completions.into_iter().collect(),
                            ));

                            res
                        }
                    });
                    return true;
                }
            }
            _ => {}
        }

        if input.handle_key(key) {
            *picker = None;
            true
        } else if let Some(picker) = picker {
            picker_op(input, picker, |picker| picker.handle_key(key))
        } else {
            false
        }
    }

    fn cursor_pos(&mut self, layout: ui::Layout) -> (u16, u16) {
        self.input.cursor_pos(layout)
    }
}

impl Prompt for InputPrompt<'_, '_> {
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
        if self.picker.is_some() {
            self.picker = None;
            return Ok(Validation::Continue);
        }

        if !self.input.has_value() {
            if self.has_default() {
                return Ok(Validation::Finish);
            } else {
                return Err("Please enter a string".into());
            }
        }

        if let Validate::Sync(ref mut validate) = self.input_opts.validate {
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

impl Input<'_> {
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
                picker: None,
                answers,
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(&ans.as_str().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::String(ans))
    }
}

pub struct InputBuilder<'a> {
    opts: Options<'a>,
    input: Input<'a>,
}

impl<'a> InputBuilder<'a> {
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

    crate::impl_options_builder!();
    crate::impl_auto_complete_builder!(String; input);
    crate::impl_filter_builder!(String; input);
    crate::impl_validate_builder!(str; input);
    crate::impl_transform_builder!(str; input);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Input(self.input))
    }
}

impl<'a> From<InputBuilder<'a>> for super::Question<'a> {
    fn from(builder: InputBuilder<'a>) -> Self {
        builder.build()
    }
}

fn remove_brackets(mut s: String) -> String {
    s.remove(0);
    s.pop();
    s
}

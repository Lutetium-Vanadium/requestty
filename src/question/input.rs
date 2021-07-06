use ui::{
    backend::Backend,
    error,
    events::{KeyCode, KeyEvent},
    style::Stylize,
    widgets, Prompt, Validation, Widget,
};

use super::{AutoComplete, ChoiceList, Completions, Filter, Options, Transform, Validate};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Input<'a> {
    default: Option<String>,
    filter: Filter<'a, String>,
    validate: Validate<'a, str>,
    transform: Transform<'a, str>,
    auto_complete: AutoComplete<'a, String>,
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

#[inline]
/// Calls a function with the given select. Anytime the select is used, it must be used
/// through this function. This is is because the selected element of the select doesn't
/// actually contain the element, it is contained by the input. This function
/// temporarily swaps the select's selected item and the input, performs the function,
/// and swaps back.
fn select_op<T, F: FnOnce(&mut CompletionSelector) -> T>(
    input: &mut widgets::StringInput,
    select: &mut CompletionSelector,
    op: F,
) -> T {
    let mut res = None;

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

    res.unwrap()
}

impl Widget for InputPrompt<'_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.prompt.render(layout, b)?;
        self.input.render(layout, b)?;
        if let Some(ref mut select) = self.select {
            select_op(&mut self.input, select, |select| select.render(layout, b))?;
        }
        Ok(())
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        let mut height = self.prompt.height(layout) + self.input.height(layout) - 1;
        if let Some(ref mut select) = self.select {
            height += select_op(&mut self.input, select, |select| select.height(layout)) - 1;
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
                    input.replace_with(|s| {
                        let mut completions = ac(s, answers);
                        assert!(!completions.is_empty());
                        if completions.len() == 1 {
                            completions.pop().unwrap()
                        } else {
                            let res = std::mem::take(&mut completions[0]);

                            *select = Some(widgets::Select::new(
                                completions.into_iter().map(widgets::Text::new).collect(),
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
            *select = None;
            true
        } else if let Some(select) = select {
            select_op(input, select, |select| select.handle_key(key))
        } else {
            false
        }
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
            .unwrap_or_else(|| prompt.into_hint().unwrap());

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

    pub(crate) fn ask<B: Backend, E: Iterator<Item = error::Result<KeyEvent>>>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> error::Result<Answer> {
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

#[cfg(test)]
mod tests {
    use ui::{backend::TestBackend, layout::Layout};

    use super::*;

    const NINPUTS: usize = 3;
    static INPUT_IDS: [&str; NINPUTS] = ["no_default", "default", "auto_complete"];
    const AUTO_COMPLETE_IDX: usize = 2;

    fn inputs(answers: &Answers) -> [(InputPrompt<'static, '_>, u16); NINPUTS] {
        [
            (Input::default().into_input_prompt("message", &answers), 17),
            (
                Input {
                    default: Some("default".into()),
                    ..Input::default()
                }
                .into_input_prompt("message", &answers),
                25,
            ),
            (
                Input {
                    auto_complete: AutoComplete::Sync(Box::new(|s, _| {
                        let mut completions: Completions<_> = ('a'..='d')
                            .map(|c| {
                                let mut s = s.clone();
                                s.push(c);
                                s
                            })
                            .collect();
                        completions.push(s + "e");
                        completions
                    })),
                    ..Input::default()
                }
                .into_input_prompt("message", &answers),
                17,
            ),
        ]
    }

    #[test]
    fn test_render() {
        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);
        let answers = Answers::default();

        let mut inputs = inputs(&answers);
        let mut backend = TestBackend::new_with_layout(size, base_layout);

        for (i, (prompt, line_offset)) in inputs.iter_mut().enumerate() {
            let line_offset = *line_offset;

            let mut layout = base_layout;
            backend.reset_with_layout(layout);
            assert!(prompt.render(&mut layout, &mut backend).is_ok());
            assert_eq!(layout, base_layout.with_line_offset(line_offset));
            ui::assert_backend_snapshot!(format!("{}-1", INPUT_IDS[i]), backend);

            prompt.input.set_value("input".repeat(10));

            layout = base_layout;
            backend.reset_with_layout(layout);
            assert!(prompt.render(&mut layout, &mut backend).is_ok());
            assert_eq!(
                layout,
                base_layout.with_offset(0, 1).with_line_offset(line_offset)
            );
            ui::assert_backend_snapshot!(format!("{}-2", INPUT_IDS[i]), backend);
        }

        let prompt = &mut inputs[AUTO_COMPLETE_IDX].0;

        prompt.input.replace_with(|mut s| {
            s.truncate(5);
            s
        });
        assert!(prompt.handle_key(KeyCode::Tab.into()));

        let mut layout = base_layout;
        backend.reset_with_layout(layout);
        assert!(prompt.render(&mut layout, &mut backend).is_ok());
        assert_eq!(layout, base_layout.with_offset(0, 6).with_line_offset(0));
        ui::assert_backend_snapshot!(format!("{}-3", INPUT_IDS[AUTO_COMPLETE_IDX]), backend);

        assert!(prompt.handle_key(KeyCode::Tab.into()));
        assert_eq!(prompt.validate(), Ok(Validation::Continue));

        layout = base_layout;
        backend.reset_with_layout(layout);
        assert!(prompt.render(&mut layout, &mut backend).is_ok());
        assert_eq!(
            layout,
            base_layout
                .with_offset(0, 0)
                .with_line_offset(inputs[AUTO_COMPLETE_IDX].1 + 6)
        );
        ui::assert_backend_snapshot!(format!("{}-4", INPUT_IDS[AUTO_COMPLETE_IDX]), backend);
    }

    #[test]
    fn test_height() {
        let size = (50, 20).into();
        let base_layout = Layout::new(5, size);
        let answers = Answers::default();

        let mut inputs = inputs(&answers);

        for (prompt, line_offset) in inputs.iter_mut() {
            let line_offset = *line_offset;

            let mut layout = base_layout;
            assert_eq!(prompt.height(&mut layout), 1);
            assert_eq!(layout, base_layout.with_line_offset(line_offset));

            prompt.input.set_value("input".repeat(10));

            layout = base_layout;
            assert_eq!(prompt.height(&mut layout), 2);
            assert_eq!(
                layout,
                base_layout.with_offset(0, 1).with_line_offset(line_offset)
            );
        }

        let prompt = &mut inputs[AUTO_COMPLETE_IDX].0;

        prompt.input.replace_with(|mut s| {
            s.truncate(5);
            s
        });
        assert!(prompt.handle_key(KeyCode::Tab.into()));

        let mut layout = base_layout;
        assert_eq!(prompt.height(&mut layout), 6);
        assert_eq!(layout, base_layout.with_offset(0, 6).with_line_offset(0));

        assert!(prompt.handle_key(KeyCode::Tab.into()));
        assert_eq!(prompt.validate(), Ok(Validation::Continue));

        layout = base_layout;
        assert_eq!(prompt.height(&mut layout), 1);
        assert_eq!(
            layout,
            base_layout
                .with_offset(0, 0)
                .with_line_offset(inputs[AUTO_COMPLETE_IDX].1 + 6)
        );
    }

    #[test]
    fn test_cursor_pos() {
        let size = (50, 20).into();
        let layout = Layout::new(5, size);
        let answers = Answers::default();

        let mut inputs = inputs(&answers);

        for (prompt, line_offset) in inputs.iter_mut() {
            let line_offset = *line_offset;

            assert_eq!(prompt.cursor_pos(layout), (line_offset, 0));
            prompt.input.set_value("input".repeat(10));
            prompt.input.set_at(50);
            assert_eq!(prompt.cursor_pos(layout), (line_offset, 1));
        }

        let prompt = &mut inputs[AUTO_COMPLETE_IDX].0;
        let line_offset = inputs[AUTO_COMPLETE_IDX].1;

        prompt.input.replace_with(|mut s| {
            s.truncate(5);
            s
        });
        assert!(prompt.handle_key(KeyCode::Tab.into()));

        assert_eq!(prompt.cursor_pos(layout), (line_offset + 6, 0));

        assert!(prompt.handle_key(KeyCode::Tab.into()));
        assert_eq!(prompt.validate(), Ok(Validation::Continue));

        assert_eq!(prompt.cursor_pos(layout), (line_offset + 6, 0));
    }
}

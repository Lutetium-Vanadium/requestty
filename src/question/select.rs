use ui::{
    backend::Backend,
    error,
    events::KeyEvent,
    style::Stylize,
    widgets::{self, Text},
    Prompt, Widget,
};

use super::{Options, Transform};
use crate::{Answer, Answers, ListItem};

#[derive(Debug, Default)]
pub struct Select<'a> {
    choices: super::ChoiceList<Text<String>>,
    transform: Transform<'a, ListItem>,
}

struct SelectPrompt<'a> {
    prompt: widgets::Prompt<&'a str>,
    select: widgets::Select<Select<'a>>,
}

impl SelectPrompt<'_> {
    fn finish_index(self, index: usize) -> ListItem {
        ListItem {
            index,
            name: self
                .select
                .into_inner()
                .choices
                .choices
                .swap_remove(index)
                .unwrap_choice()
                .text,
        }
    }
}

impl Prompt for SelectPrompt<'_> {
    type ValidateErr = &'static str;
    type Output = ListItem;

    fn finish(self) -> Self::Output {
        let index = self.select.get_at();
        self.finish_index(index)
    }
}

impl Widget for SelectPrompt<'_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.prompt.render(layout, b)?;
        self.select.render(layout, b)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.select.height(layout) - 1
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.select.cursor_pos(layout)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.select.handle_key(key)
    }
}

impl widgets::List for Select<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        layout: ui::layout::Layout,
        backend: &mut B,
    ) -> error::Result<()> {
        self.choices.render_item(index, hovered, layout, backend)
    }

    fn is_selectable(&self, index: usize) -> bool {
        self.choices.is_selectable(index)
    }

    fn height_at(&mut self, index: usize, layout: ui::layout::Layout) -> u16 {
        self.choices.height_at(index, layout)
    }

    fn len(&self) -> usize {
        self.choices.len()
    }

    fn page_size(&self) -> usize {
        self.choices.page_size()
    }

    fn should_loop(&self) -> bool {
        self.choices.should_loop()
    }
}

impl<'a> Select<'a> {
    fn into_prompt(self, message: &'a str) -> SelectPrompt<'a> {
        let mut select = widgets::Select::new(self);
        if let Some(default) = select.list.choices.default() {
            select.set_at(default);
        }

        SelectPrompt {
            prompt: widgets::Prompt::new(message),
            select,
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
        let ans = ui::Input::new(self.into_prompt(&message), b)
            .hide_cursor()
            .run(events)?;

        crate::write_final!(
            transform,
            message,
            &ans,
            answers,
            b,
            b.write_styled(&ans.name.lines().next().unwrap().cyan())?
        );

        Ok(Answer::ListItem(ans))
    }
}

pub struct SelectBuilder<'a> {
    opts: Options<'a>,
    list: Select<'a>,
}

impl<'a> SelectBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        SelectBuilder {
            opts: Options::new(name),
            list: Default::default(),
        }
    }

    pub fn default(mut self, default: usize) -> Self {
        self.list.choices.set_default(default);
        self
    }

    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::Separator(text.into()));
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::DefaultSeparator);
        self
    }

    pub fn choice<I: Into<String>>(mut self, choice: I) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::Choice(Text::new(choice.into())));
        self
    }

    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<super::Choice<String>>,
        I: IntoIterator<Item = T>,
    {
        self.list.choices.choices.extend(
            choices
                .into_iter()
                .map(|choice| choice.into().map(Text::new)),
        );
        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.list.choices.set_page_size(page_size);
        self
    }

    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.list.choices.set_should_loop(should_loop);
        self
    }

    crate::impl_options_builder!();
    crate::impl_transform_builder!(ListItem; list);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Select(self.list))
    }
}

impl<'a> From<SelectBuilder<'a>> for super::Question<'a> {
    fn from(builder: SelectBuilder<'a>) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use rand_chacha::ChaCha12Rng;
    use ui::{backend::TestBackend, events::KeyCode, layout::Layout};

    use crate::question::{Choice, Question, QuestionKind};

    use super::*;

    const SEED: u64 = 9828123;
    const SEP_RATIO: f32 = 0.3;
    const DEFAULT_SEP_RATIO: f32 = 0.10;

    fn choices(len: usize) -> impl Iterator<Item = Choice<String>> {
        let mut rng = ChaCha12Rng::seed_from_u64(SEED);

        (0..len).map(move |i| {
            let rand: f32 = rng.gen();
            if rand < DEFAULT_SEP_RATIO {
                Choice::DefaultSeparator
            } else if rand < SEP_RATIO {
                Choice::Separator(format!("Separator {}", i))
            } else {
                Choice::Choice(format!("Choice {}", i))
            }
        })
    }

    fn unwrap_select<'a>(question: impl Into<Question<'a>>) -> Select<'a> {
        match question.into().kind {
            QuestionKind::Select(s) => s,
            _ => unreachable!(),
        }
    }

    macro_rules! test_select {
        ($mod_name:ident { select = $select:expr; height = $height:expr $(;)? }) => {
            test_select!($mod_name {
                select = $select;
                height = $height;
                events = [
                    KeyEvent::from(KeyCode::PageDown),
                    KeyCode::Up.into(),
                ];
            });
        };

        ($mod_name:ident { select = $select:expr; height = $height:expr; events = $events:expr $(;)? }) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn test_height() {
                    let size = (50, 20).into();
                    let base_layout = Layout::new(5, size);
                    let mut select = $select.into_prompt("message");

                    let events = $events;

                    for &key in events.iter() {
                        let mut layout = base_layout;

                        assert_eq!(select.height(&mut layout), $height);
                        assert_eq!(
                            layout,
                            base_layout.with_offset(0, $height).with_line_offset(0)
                        );

                        assert!(select.handle_key(key))
                    }

                    let mut layout = base_layout;

                    assert_eq!(select.height(&mut layout), $height);
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );
                }

                #[test]
                fn test_render() {
                    let size = (50, 20).into();
                    let base_layout = Layout::new(5, size);
                    let mut select = $select.into_prompt("message");

                    let mut backend = TestBackend::new(size);

                    let events = $events;

                    for &key in events.iter() {
                        let mut layout = base_layout;
                        backend.reset_with_layout(layout);

                        assert!(select.render(&mut layout, &mut backend).is_ok());
                        assert_eq!(
                            layout,
                            base_layout.with_offset(0, $height).with_line_offset(0)
                        );
                        ui::assert_backend_snapshot!(backend);

                        assert!(select.handle_key(key))
                    }

                    let mut layout = base_layout;
                    backend.reset_with_layout(layout);

                    assert!(select.render(&mut layout, &mut backend).is_ok());
                    assert_eq!(
                        layout,
                        base_layout.with_offset(0, $height).with_line_offset(0)
                    );
                    ui::assert_backend_snapshot!(backend);
                }
            }
        };
    }

    test_select!(basic {
        select = unwrap_select(
                SelectBuilder::new("name".into()).choices(choices(10)),
            );
        height = 11;
    });

    test_select!(pagination {
        select = unwrap_select(
                SelectBuilder::new("name".into()).choices(choices(20)).default(6),
            );
        height = 16;
    });
}

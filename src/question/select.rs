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
                .finish()
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

    fn has_default(&self) -> bool {
        self.select.list.choices.default().is_some()
    }
    fn finish_default(self) -> Self::Output {
        let index = self.select.list.choices.default().unwrap();
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

impl Select<'_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();
        let mut select = widgets::Select::new(self);
        if let Some(default) = select.list.choices.default() {
            select.set_at(default);
        }
        let ans = ui::Input::new(
            SelectPrompt {
                prompt: widgets::Prompt::new(&message),
                select,
            },
            b,
        )
        .hide_cursor()
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                widgets::Prompt::write_finished_message(&message, b)?;
                b.write_styled(&ans.name.lines().next().unwrap().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

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
        self.list
            .choices
            .choices
            .extend(choices.into_iter().map(|choice| match choice.into() {
                super::Choice::Choice(c) => super::Choice::Choice(Text::new(c)),
                super::Choice::Separator(s) => super::Choice::Separator(s),
                super::Choice::DefaultSeparator => super::Choice::DefaultSeparator,
            }));
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

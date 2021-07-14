use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyEvent},
    style::Stylize,
    widgets::{self, Text},
    Prompt, Widget,
};

use super::Transform;
use crate::{Answer, Answers, ListItem};

pub use builder::SelectBuilder;

mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct Select<'a> {
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
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
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
    ) -> io::Result<()> {
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

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Answer> {
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
            b.write_styled(
                &ans.name
                    .lines()
                    .next()
                    .expect("There must be at least one line in a `str`")
                    .cyan()
            )?
        );

        Ok(Answer::ListItem(ans))
    }
}

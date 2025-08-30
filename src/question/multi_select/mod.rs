use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyCode, KeyEvent},
    style::Color,
    widgets::{self, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Filter, Transform, Validate};
use crate::{Answer, Answers, ListItem};

pub use builder::MultiSelectBuilder;

mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct MultiSelect<'a> {
    choices: super::ChoiceList<Text<String>>,
    selected: Vec<bool>,
    filter: Filter<'a, Vec<bool>>,
    validate: Validate<'a, [bool]>,
    transform: Transform<'a, [ListItem]>,
}

fn set_seperators_false(selected: &mut [bool], choices: &[Choice<Text<String>>]) {
    for (i, choice) in choices.iter().enumerate() {
        selected[i] &= !choice.is_separator();
    }
}

struct MultiSelectPrompt<'a, 'c> {
    prompt: widgets::Prompt<&'a str>,
    select: widgets::Select<MultiSelect<'c>>,
    answers: &'a Answers,
}

fn create_list_items(
    selected: Vec<bool>,
    choices: super::ChoiceList<Text<String>>,
) -> Vec<ListItem> {
    selected
        .into_iter()
        .enumerate()
        .zip(choices.choices)
        .filter_map(|((index, is_selected), text)| match (is_selected, text) {
            (true, Choice::Choice(text)) => Some(ListItem {
                index,
                text: text.text,
            }),
            _ => None,
        })
        .collect()
}

impl Prompt for MultiSelectPrompt<'_, '_> {
    type ValidateErr = widgets::Text<String>;
    type Output = Vec<ListItem>;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Sync(ref mut validate) = self.select.list.validate {
            set_seperators_false(
                &mut self.select.list.selected,
                &self.select.list.choices.choices,
            );
            validate(&self.select.list.selected, self.answers)?;
        }
        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        let MultiSelect {
            mut selected,
            choices,
            filter,
            ..
        } = self.select.into_inner();

        if let Filter::Sync(filter) = filter {
            set_seperators_false(&mut selected, &choices.choices);

            selected = filter(selected, self.answers);
        }

        create_list_items(selected, choices)
    }
}

impl Widget for MultiSelectPrompt<'_, '_> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        self.prompt.render(layout, b)?;
        self.select.render(layout, b)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.select.height(layout) - 1
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(' ') => {
                let index = self.select.get_at();
                self.select.list.selected[index] = !self.select.list.selected[index];
            }
            KeyCode::Char('i') => {
                self.select.list.selected.iter_mut().for_each(|s| *s = !*s);
            }
            KeyCode::Char('a') => {
                let select_state = self.select.list.selected.iter().any(|s| !s);
                self.select
                    .list
                    .selected
                    .iter_mut()
                    .for_each(|s| *s = select_state);
            }
            _ => return self.select.handle_key(key),
        }

        true
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.select.cursor_pos(layout)
    }
}

impl widgets::List for MultiSelect<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> io::Result<()> {
        let symbol_set = ui::symbols::current();
        if hovered {
            b.set_fg(Color::Cyan)?;
            write!(b, "{} ", symbol_set.pointer)?;
        } else {
            b.write_all(b"  ")?;
        }

        if self.is_selectable(index) {
            if self.selected[index] {
                b.set_fg(Color::LightGreen)?;
                write!(b, "{} ", symbol_set.completed)?;
            } else {
                b.set_fg(Color::DarkGrey)?;
                write!(b, "  ")?;
            }


            if hovered {
                b.set_fg(Color::Cyan)?;
            } else {
                b.set_fg(Color::Reset)?;
            }
        } else {
            b.set_fg(Color::DarkGrey)?;
        }

        layout.offset_x += 4;

        self.choices[index].render(&mut layout, b)?;

        b.set_fg(Color::Reset)
    }

    fn is_selectable(&self, index: usize) -> bool {
        !self.choices[index].is_separator()
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        layout.offset_x += 4;
        self.choices[index].height(&mut layout)
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

impl<'c> MultiSelect<'c> {
    fn into_multi_select_prompt<'a>(
        self,
        message: &'a str,
        answers: &'a Answers,
    ) -> MultiSelectPrompt<'a, 'c> {
        MultiSelectPrompt {
            prompt: widgets::Prompt::new(message)
                .with_hint("Press <space> to select, <a> to toggle all, <i> to invert selection"),
            select: widgets::Select::new(self),
            answers,
        }
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        on_esc: ui::OnEsc,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Option<Answer>> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_multi_select_prompt(&message, answers), b)
            .hide_cursor()
            .on_esc(on_esc)
            .run(events)?;

        crate::write_final!(transform, message, ans [ref], answers, b, |ans| {
            b.set_fg(Color::Cyan)?;
            print_comma_separated(
                ans.iter().map(|item| {
                    item.text
                        .lines()
                        .next()
                        .expect("There must be at least one line in a `str`")
                }),
                b,
            )?;
            b.set_fg(Color::Reset)?;
        })
    }
}

fn print_comma_separated<'a, B: Backend>(
    iter: impl Iterator<Item = &'a str>,
    b: &mut B,
) -> io::Result<()> {
    let mut iter = iter.peekable();

    while let Some(item) = iter.next() {
        b.write_all(item.as_bytes())?;
        if iter.peek().is_some() {
            b.write_all(b", ")?;
        }
    }

    Ok(())
}

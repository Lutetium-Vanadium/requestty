use std::io;

use ui::{
    backend::Backend,
    error,
    events::{KeyCode, KeyEvent},
    style::Color,
    widgets::{self, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Filter, Options, Transform, Validate};
use crate::{Answer, Answers, ListItem};

#[derive(Debug, Default)]
pub struct Checkbox<'a> {
    choices: super::ChoiceList<Text<String>>,
    selected: Vec<bool>,
    filter: Filter<'a, Vec<bool>>,
    validate: Validate<'a, [bool]>,
    transform: Transform<'a, [ListItem]>,
}

struct CheckboxPrompt<'a, 'c> {
    prompt: widgets::Prompt<&'a str>,
    select: widgets::Select<Checkbox<'c>>,
    answers: &'a Answers,
}

fn create_list_items(
    selected: Vec<bool>,
    choices: super::ChoiceList<Text<String>>,
) -> Vec<ListItem> {
    selected
        .into_iter()
        .enumerate()
        .zip(choices.choices.into_iter())
        .filter_map(|((index, is_selected), name)| match (is_selected, name) {
            (true, Choice::Choice(name)) => Some(ListItem {
                index,
                name: name.text,
            }),
            _ => None,
        })
        .collect()
}

impl Prompt for CheckboxPrompt<'_, '_> {
    type ValidateErr = String;
    type Output = Vec<ListItem>;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Sync(ref mut validate) = self.select.list.validate {
            validate(&self.select.list.selected, self.answers)?;
        }
        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        let Checkbox {
            mut selected,
            choices,
            filter,
            ..
        } = self.select.finish();

        if let Filter::Sync(filter) = filter {
            selected = filter(selected, self.answers);
        }

        create_list_items(selected, choices)
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Widget for CheckboxPrompt<'_, '_> {
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

impl widgets::List for Checkbox<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        if hovered {
            b.set_fg(Color::Cyan)?;
            write!(b, "{} ", ui::symbols::ARROW)?;
        } else {
            b.write_all(b"  ")?;
        }

        if self.is_selectable(index) {
            if self.selected[index] {
                b.set_fg(Color::LightGreen)?;
            } else {
                b.set_fg(Color::DarkGrey)?;
            }

            write!(b, "{} ", ui::symbols::TICK)?;

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

impl Checkbox<'_> {
    pub(crate) fn ask<B: Backend, E: Iterator<Item = error::Result<KeyEvent>>>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(
            CheckboxPrompt {
                prompt: widgets::Prompt::new(&*message)
                    .with_hint("Press <space> to select, <a> to toggle all, <i> to invert selection"),
                select: widgets::Select::new(self),
                answers,
            },
            b,
        )
        .hide_cursor()
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                widgets::Prompt::write_finished_message(&message, b)?;

                b.set_fg(Color::Cyan)?;
                print_comma_separated(
                    ans.iter().map(|item| item.name.lines().next().unwrap()),
                    b,
                )?;
                b.set_fg(Color::Reset)?;

                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::ListItems(ans))
    }
}

pub struct CheckboxBuilder<'a> {
    opts: Options<'a>,
    checkbox: Checkbox<'a>,
}

impl<'a> CheckboxBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        CheckboxBuilder {
            opts: Options::new(name),
            checkbox: Default::default(),
        }
    }

    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.checkbox
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self.checkbox.selected.push(false);
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.checkbox.choices.choices.push(Choice::DefaultSeparator);
        self.checkbox.selected.push(false);
        self
    }

    pub fn choice<I: Into<String>>(self, choice: I) -> Self {
        self.choice_with_default(choice, false)
    }

    pub fn choice_with_default<I: Into<String>>(
        mut self,
        choice: I,
        default: bool,
    ) -> Self {
        self.checkbox
            .choices
            .choices
            .push(Choice::Choice(Text::new(choice.into())));
        self.checkbox.selected.push(default);
        self
    }

    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<String>>,
        I: IntoIterator<Item = T>,
    {
        self.checkbox
            .choices
            .choices
            .extend(choices.into_iter().map(|c| match c.into() {
                Choice::Choice(c) => Choice::Choice(Text::new(c)),
                Choice::Separator(s) => Choice::Separator(s),
                Choice::DefaultSeparator => Choice::DefaultSeparator,
            }));
        self.checkbox
            .selected
            .resize(self.checkbox.choices.len(), false);
        self
    }

    pub fn choices_with_default<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<(String, bool)>>,
        I: IntoIterator<Item = T>,
    {
        let iter = choices.into_iter();
        self.checkbox
            .selected
            .reserve(iter.size_hint().0.saturating_add(1));
        self.checkbox
            .choices
            .choices
            .reserve(iter.size_hint().0.saturating_add(1));

        for choice in iter {
            match choice.into() {
                Choice::Choice((choice, selected)) => {
                    self.checkbox
                        .choices
                        .choices
                        .push(Choice::Choice(Text::new(choice)));
                    self.checkbox.selected.push(selected);
                }
                Choice::Separator(s) => {
                    self.checkbox.choices.choices.push(Choice::Separator(s));
                    self.checkbox.selected.push(false);
                }
                Choice::DefaultSeparator => {
                    self.checkbox.choices.choices.push(Choice::DefaultSeparator);
                    self.checkbox.selected.push(false);
                }
            }
        }
        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.checkbox.choices.set_page_size(page_size);
        self
    }

    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.checkbox.choices.set_should_loop(should_loop);
        self
    }

    crate::impl_options_builder!();
    crate::impl_filter_builder!(Vec<bool>; checkbox);
    crate::impl_validate_builder!([bool]; checkbox);
    crate::impl_transform_builder!([ListItem]; checkbox);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Checkbox(self.checkbox))
    }
}

impl<'a> From<CheckboxBuilder<'a>> for super::Question<'a> {
    fn from(builder: CheckboxBuilder<'a>) -> Self {
        builder.build()
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

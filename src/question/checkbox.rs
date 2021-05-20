use std::io;

use ui::{
    backend::{Backend, Color},
    error,
    events::{KeyCode, KeyEvent},
    widgets::{self, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Filter, Options, Transform, Validate};
use crate::{Answer, Answers, ListItem};

#[derive(Debug, Default)]
pub struct Checkbox<'f, 'v, 't> {
    choices: super::ChoiceList<Text<String>>,
    selected: Vec<bool>,
    filter: Filter<'f, Vec<bool>>,
    validate: Validate<'v, [bool]>,
    transform: Transform<'t, [ListItem]>,
}

struct CheckboxPrompt<'f, 'v, 't, 'a> {
    message: String,
    picker: widgets::ListPicker<Checkbox<'f, 'v, 't>>,
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

impl Prompt for CheckboxPrompt<'_, '_, '_, '_> {
    type ValidateErr = String;
    type Output = Vec<ListItem>;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("(Press <space> to select, <a> to toggle all, <i> to invert selection)")
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Sync(ref validate) = self.picker.list.validate {
            validate(&self.picker.list.selected, self.answers)?;
        }
        Ok(Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        let Checkbox {
            mut selected,
            choices,
            filter,
            ..
        } = self.picker.finish();

        if let Filter::Sync(filter) = filter {
            selected = filter(selected, self.answers);
        }

        create_list_items(selected, choices)
    }

    fn has_default(&self) -> bool {
        false
    }
}

crate::cfg_async! {
#[async_trait::async_trait]
impl ui::AsyncPrompt for CheckboxPrompt<'_, '_, '_, '_> {
    async fn finish_async(self) -> Self::Output {
        let Checkbox {
            mut selected,
            choices,
            filter,
            ..
        } = self.picker.finish();

        selected = match filter {
            Filter::Async(filter) => filter(selected, self.answers).await,
            Filter::Sync(filter) => filter(selected, self.answers),
            Filter::None => selected,
        };

        create_list_items(selected, choices)
    }

    fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
        match self.picker.list.validate {
            Validate::Sync(ref validate) => {
                Some(validate(&self.picker.list.selected, self.answers).map(|_| Validation::Finish))
            }
            _ => None,
        }
    }

    async fn validate_async(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Validate::Async(ref validate) = self.picker.list.validate {
            validate(&self.picker.list.selected, self.answers).await?;
        }
        Ok(Validation::Finish)
    }
}
}

impl Widget for CheckboxPrompt<'_, '_, '_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: ui::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.picker.render(layout, b)
    }

    fn height(&mut self, layout: ui::Layout) -> u16 {
        self.picker.height(layout)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(' ') => {
                let index = self.picker.get_at();
                self.picker.list.selected[index] = !self.picker.list.selected[index];
            }
            KeyCode::Char('i') => {
                self.picker.list.selected.iter_mut().for_each(|s| *s = !*s);
            }
            KeyCode::Char('a') => {
                let select_state = self.picker.list.selected.iter().any(|s| !s);
                self.picker
                    .list
                    .selected
                    .iter_mut()
                    .for_each(|s| *s = select_state);
            }
            _ => return self.picker.handle_key(key),
        }

        true
    }

    fn cursor_pos(&mut self, layout: ui::Layout) -> (u16, u16) {
        self.picker.cursor_pos(layout)
    }
}

impl widgets::List for Checkbox<'_, '_, '_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        if hovered {
            b.set_fg(Color::Cyan)?;
            b.write_all("❯ ".as_bytes())?;
        } else {
            b.write_all(b"  ")?;
        }

        if self.is_selectable(index) {
            if self.selected[index] {
                b.set_fg(Color::LightGreen)?;
                b.write_all("◉ ".as_bytes())?;

                if hovered {
                    b.set_fg(Color::Cyan)?;
                } else {
                    b.set_fg(Color::Reset)?;
                }
            } else {
                b.write_all("◯ ".as_bytes())?;
            }
        } else {
            b.set_fg(Color::DarkGrey)?;
        }

        layout.offset_x += 4;

        self.choices[index].render(layout, b)?;

        b.set_fg(Color::Reset)
    }

    fn is_selectable(&self, index: usize) -> bool {
        !self.choices[index].is_separator()
    }

    fn height_at(&mut self, index: usize, layout: ui::Layout) -> u16 {
        self.choices[index].height(layout)
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

impl Checkbox<'_, '_, '_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(
            CheckboxPrompt {
                message,
                picker: widgets::ListPicker::new(self),
                answers,
            },
            b,
        )
        .hide_cursor()
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
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

    crate::cfg_async! {
    pub(crate) async fn ask_async<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::AsyncEvents,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(
            CheckboxPrompt {
                message,
                picker: widgets::ListPicker::new(self),
                answers,
            },
            b,
        )
        .hide_cursor()
        .run_async(events)
        .await?;

        match transform {
            Transform::Async(transform) => transform(&ans, answers, b).await?,
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
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
}

pub struct CheckboxBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    checkbox: Checkbox<'f, 'v, 't>,
}

impl<'m, 'w, 'f, 'v, 't> CheckboxBuilder<'m, 'w, 'f, 'v, 't> {
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

    pub fn build(self) -> super::Question<'m, 'w, 'f, 'v, 't> {
        super::Question::new(self.opts, super::QuestionKind::Checkbox(self.checkbox))
    }
}

impl<'m, 'w, 'f, 'v, 't> From<CheckboxBuilder<'m, 'w, 'f, 'v, 't>>
    for super::Question<'m, 'w, 'f, 'v, 't>
{
    fn from(builder: CheckboxBuilder<'m, 'w, 'f, 'v, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(CheckboxBuilder<'f, 'v, 't>; (this, opts) => {
    CheckboxBuilder {
        opts,
        checkbox: this.checkbox,
    }
});

crate::impl_filter_builder!(CheckboxBuilder<'m, 'w, f, 'v, 't> Vec<bool>; (this, filter) => {
    CheckboxBuilder {
        opts: this.opts,
        checkbox: Checkbox {
            filter,
            choices: this.checkbox.choices,
            validate: this.checkbox.validate,
            transform: this.checkbox.transform,
            selected: this.checkbox.selected,
        }
    }
});

crate::impl_validate_builder!(CheckboxBuilder<'m, 'w, 'f, v, 't> [bool]; (this, validate) => {
    CheckboxBuilder {
        opts: this.opts,
        checkbox: Checkbox {
            validate,
            choices: this.checkbox.choices,
            filter: this.checkbox.filter,
            transform: this.checkbox.transform,
            selected: this.checkbox.selected,
        }
    }
});

crate::impl_transform_builder!(CheckboxBuilder<'m, 'w, 'f, 'v, t> [ListItem]; (this, transform) => {
    CheckboxBuilder {
        opts: this.opts,
        checkbox: Checkbox {
            transform,
            choices: this.checkbox.choices,
            filter: this.checkbox.filter,
            validate: this.checkbox.validate,
            selected: this.checkbox.selected,
        }
    }
});

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

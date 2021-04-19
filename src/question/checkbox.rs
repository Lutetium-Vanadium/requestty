use std::{fmt, io};

use crossterm::{
    event, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use ui::{widgets, Validation, Widget};

use crate::{error, Answer, Answers, ListItem};

use super::{none, some, Choice, Filter, Options, Transformer, Validate};

#[derive(Default)]
pub struct Checkbox<'f, 'v, 't> {
    choices: super::ChoiceList<String>,
    selected: Vec<bool>,
    filter: Option<Box<Filter<'f, Vec<bool>>>>,
    validate: Option<Box<Validate<'v, [bool]>>>,
    transformer: Option<Box<Transformer<'t, [ListItem]>>>,
}

impl fmt::Debug for Checkbox<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Checkbox")
            .field("choices", &self.choices)
            .field("selected", &self.selected)
            .field("filter", &self.filter.as_ref().map_or_else(none, some))
            .field("validate", &self.validate.as_ref().map_or_else(none, some))
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
}

struct CheckboxPrompt<'f, 'v, 't, 'a> {
    message: String,
    picker: widgets::ListPicker<Checkbox<'f, 'v, 't>>,
    answers: &'a Answers,
}

impl ui::Prompt for CheckboxPrompt<'_, '_, '_, '_> {
    type ValidateErr = String;
    type Output = Vec<ListItem>;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("(Press <space> to select, <a> to toggle all, <i> to invert selection)")
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if let Some(ref validate) = self.picker.list.validate {
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

        if let Some(filter) = filter {
            selected = filter(selected, self.answers);
        }

        selected
            .into_iter()
            .enumerate()
            .zip(choices.choices.into_iter())
            .filter_map(|((index, is_selected), name)| match (is_selected, name) {
                (true, Choice::Choice(name)) => Some(ListItem { index, name }),
                _ => None,
            })
            .collect()
    }

    fn has_default(&self) -> bool {
        false
    }
}

impl Widget for CheckboxPrompt<'_, '_, '_, '_> {
    fn render<W: io::Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        self.picker.render(max_width, w)
    }

    fn height(&self) -> usize {
        self.picker.height()
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        match key.code {
            event::KeyCode::Char(' ') => {
                let index = self.picker.get_at();
                self.picker.list.selected[index] = !self.picker.list.selected[index];
            }
            event::KeyCode::Char('i') => {
                self.picker.list.selected.iter_mut().for_each(|s| *s = !*s);
            }
            event::KeyCode::Char('a') => {
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

    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        self.picker.cursor_pos(prompt_len)
    }
}

impl widgets::List for Checkbox<'_, '_, '_> {
    fn render_item<W: io::Write>(
        &mut self,
        index: usize,
        hovered: bool,
        max_width: usize,
        w: &mut W,
    ) -> crossterm::Result<()> {
        if hovered {
            queue!(w, SetForegroundColor(Color::DarkCyan), Print("❯ "))?;
        } else {
            w.write_all(b"  ")?;
        }

        if self.is_selectable(index) {
            if self.selected[index] {
                queue!(w, SetForegroundColor(Color::Green), Print("◉ "),)?;

                if hovered {
                    queue!(w, SetForegroundColor(Color::DarkCyan))?;
                } else {
                    queue!(w, ResetColor)?;
                }
            } else {
                w.write_all("◯ ".as_bytes())?;
            }
        } else {
            queue!(w, SetForegroundColor(Color::DarkGrey))?;
        }

        self.choices[index].as_str().render(max_width - 4, w)?;

        queue!(w, ResetColor)
    }

    fn is_selectable(&self, index: usize) -> bool {
        !self.choices[index].is_separator()
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
    pub fn ask<W: io::Write>(
        mut self,
        message: String,
        answers: &Answers,
        w: &mut W,
    ) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let ans = ui::Input::new(CheckboxPrompt {
            message,
            picker: widgets::ListPicker::new(self),
            answers,
        })
        .hide_cursor()
        .run(w)?;

        match transformer {
            Some(transformer) => transformer(&ans, answers, w)?,
            None => {
                queue!(w, SetForegroundColor(Color::DarkCyan))?;
                print_comma_separated(ans.iter().map(|item| item.name.as_str()), w)?;

                w.write_all(b"\n")?;
                execute!(w, ResetColor)?;
            }
        }

        Ok(Answer::ListItems(ans))
    }
}

pub struct CheckboxBuilder<'m, 'w, 'f, 'v, 't> {
    opts: Options<'m, 'w>,
    checkbox: Checkbox<'f, 'v, 't>,
}

impl<'m, 'w, 'f, 'v, 't> CheckboxBuilder<'m, 'w, 'f, 'v, 't> {
    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.checkbox
            .choices
            .choices
            .push(Choice::Separator(Some(text.into())));
        self.checkbox.selected.push(false);
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.checkbox.choices.choices.push(Choice::Separator(None));
        self.checkbox.selected.push(false);
        self
    }

    pub fn choice<I: Into<String>>(self, choice: I) -> Self {
        self.choice_with_default(choice, false)
    }

    pub fn choice_with_default<I: Into<String>>(mut self, choice: I, default: bool) -> Self {
        self.checkbox
            .choices
            .choices
            .push(Choice::Choice(choice.into()));
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
            .extend(choices.into_iter().map(Into::into));
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
                    self.checkbox.choices.choices.push(Choice::Choice(choice));
                    self.checkbox.selected.push(selected);
                }
                Choice::Separator(s) => {
                    self.checkbox.choices.choices.push(Choice::Separator(s));
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
            transformer: this.checkbox.transformer,
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
            transformer: this.checkbox.transformer,
            selected: this.checkbox.selected,
        }
    }
});

crate::impl_transformer_builder!(CheckboxBuilder<'m, 'w, 'f, 'v, t> [ListItem]; (this, transformer) => {
    CheckboxBuilder {
        opts: this.opts,
        checkbox: Checkbox {
            transformer,
            choices: this.checkbox.choices,
            filter: this.checkbox.filter,
            validate: this.checkbox.validate,
            selected: this.checkbox.selected,
        }
    }
});

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn checkbox<N: Into<String>>(
        name: N,
    ) -> CheckboxBuilder<'static, 'static, 'static, 'static, 'static> {
        CheckboxBuilder {
            opts: Options::new(name.into()),
            checkbox: Default::default(),
        }
    }
}

fn print_comma_separated<'a, W: io::Write>(
    iter: impl Iterator<Item = &'a str>,
    w: &mut W,
) -> io::Result<()> {
    let mut iter = iter.peekable();

    while let Some(item) = iter.next() {
        w.write_all(item.as_bytes())?;
        if iter.peek().is_some() {
            w.write_all(b", ")?;
        }
    }

    Ok(())
}

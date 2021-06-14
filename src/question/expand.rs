#[cfg(feature = "ahash")]
use ahash::AHashSet as HashSet;
#[cfg(not(feature = "ahash"))]
use std::collections::HashSet;

use ui::{
    backend::{Backend, MoveDirection},
    error,
    events::KeyEvent,
    style::{Color, Stylize},
    widgets::{self, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Options, Transform};
use crate::{Answer, Answers, ExpandItem};

#[derive(Debug)]
pub struct Expand<'a> {
    choices: super::ChoiceList<ExpandItem<Text<String>>>,
    selected: Option<char>,
    default: char,
    transform: Transform<'a, ExpandItem<String>>,
}

impl<'a> Default for Expand<'a> {
    fn default() -> Self {
        Expand {
            default: 'h',
            selected: None,
            choices: Default::default(),
            transform: Transform::None,
        }
    }
}

struct ExpandPrompt<'a, F> {
    prompt: widgets::Prompt<&'a str, &'a str>,
    select: widgets::Select<Expand<'a>>,
    input: widgets::CharInput<F>,
    expanded: bool,
}

impl<F: Fn(char) -> Option<char>> ExpandPrompt<'_, F> {
    fn selected(&mut self) -> Option<&mut ExpandItem<Text<String>>> {
        let key = self.input.value()?;

        self.select
            .list
            .choices
            .choices
            .iter_mut()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) => Some(choice),
                _ => None,
            })
            .find(|item| item.key == key)
    }

    fn finish_with(self, c: char) -> ExpandItem<String> {
        let item = self
            .select
            .finish()
            .choices
            .choices
            .into_iter()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) => Some(choice),
                _ => None,
            })
            .find(|item| item.key == c)
            .unwrap();

        ExpandItem {
            name: item.name.text,
            key: item.key,
        }
    }
}

impl<F: Fn(char) -> Option<char>> Prompt for ExpandPrompt<'_, F> {
    type ValidateErr = &'static str;
    type Output = ExpandItem<String>;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        match self.input.value().unwrap_or(self.select.list.default) {
            'h' => {
                self.expanded = true;
                self.input.set_value(None);
                self.select.list.selected = None;
                Ok(Validation::Continue)
            }
            _ => Ok(Validation::Finish),
        }
    }

    fn finish(self) -> Self::Output {
        let c = self.input.value().unwrap_or(self.select.list.default);
        self.finish_with(c)
    }

    fn has_default(&self) -> bool {
        self.select.list.default != 'h'
    }

    fn finish_default(self) -> Self::Output {
        let c = self.select.list.default;
        self.finish_with(c)
    }
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl<F: Fn(char) -> Option<char>> ui::Widget for ExpandPrompt<'_, F> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.prompt.render(layout, b)?;
        if self.expanded {
            self.select.render(layout, b)?;
            b.write_all(ANSWER_PROMPT)?;
            layout.line_offset = ANSWER_PROMPT.len() as u16;
            self.input.render(layout, b)
        } else {
            self.input.render(layout, b)?;

            if self.input.value().is_some() {
                b.move_cursor(MoveDirection::NextLine(1))?;
                b.write_styled(&">> ".cyan())?;

                layout.offset_y += 1;
                layout.offset_x += 3;

                match self.selected() {
                    Some(item) => item.render(layout, b)?,
                    None => b.write_all(b"Help, list all options")?,
                }
            }

            Ok(())
        }
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        if self.expanded {
            // Don't need to add 1 for the answer prompt, since this will over count by 1 anyways
            let height = self.prompt.height(layout) + self.select.height(layout);
            layout.offset_y += 1; // For the ANSWER prompt at the bottom
            height
        } else if self.input.value().is_some() {
            self.prompt.height(layout) - 1
                + self.input.height(layout)
                // selected will return None if the help option is selected
                + self.selected().map(|c| c.height(layout)).unwrap_or(1)
        } else {
            self.prompt.height(layout) + self.input.height(layout) - 1
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.input.handle_key(key) {
            self.select.list.selected = self.input.value();
            true
        } else if self.expanded {
            self.select.handle_key(key)
        } else {
            false
        }
    }

    fn cursor_pos(&mut self, mut layout: ui::layout::Layout) -> (u16, u16) {
        if self.expanded {
            let w = self
                .input
                .cursor_pos(layout.with_line_offset(ANSWER_PROMPT.len() as u16))
                .0;
            (w, self.height(&mut layout) - 1)
        } else {
            self.input
                .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
        }
    }
}

impl widgets::List for Expand<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        _: bool,
        layout: ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        if index == self.choices.len() {
            return self.render_choice(None, layout, b);
        }

        match &mut self.choices[index] {
            Choice::Choice(_) => self.render_choice(Some(index), layout, b),
            separator => {
                b.set_fg(Color::DarkGrey)?;
                b.write_all(b"   ")?;
                super::get_sep_str(separator)
                    .render(&mut layout.with_line_offset(3), b)?;
                b.set_fg(Color::Reset)
            }
        }
    }

    fn is_selectable(&self, _: usize) -> bool {
        true
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        if index >= self.choices.len() {
            // Help option
            1
        } else {
            layout.offset_x += 5;
            self.choices[index].height(&mut layout)
        }
    }

    fn len(&self) -> usize {
        self.choices.len() + 1
    }

    fn page_size(&self) -> usize {
        self.choices.page_size()
    }

    fn should_loop(&self) -> bool {
        self.choices.should_loop()
    }
}

impl Expand<'_> {
    fn render_choice<B: Backend>(
        &mut self,
        index: Option<usize>,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        let key = match index {
            Some(index) => self.choices[index].as_ref().unwrap_choice().key,
            None => 'h',
        };

        let hovered = self.selected.map(|c| c == key).unwrap_or(false);

        if hovered {
            b.set_fg(Color::Cyan)?;
        }

        write!(b, "  {}) ", key)?;

        layout.offset_x += 5;

        match index {
            Some(index) => self.choices[index]
                .as_mut()
                .unwrap_choice()
                .render(&mut layout, b)?,
            None => "Help, list all options"
                .render(&mut layout.with_line_offset(5), b)?,
        }

        if hovered {
            b.set_fg(Color::Reset)?;
        }

        Ok(())
    }

    pub(crate) fn ask<B: Backend, E: Iterator<Item = error::Result<KeyEvent>>>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> error::Result<Answer> {
        let help_key = if self.default == 'h' { 'H' } else { 'h' };

        let hint: String = self
            .choices
            .choices
            .iter()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) if self.default == choice.key => {
                    Some(choice.key.to_ascii_uppercase())
                }
                Choice::Choice(choice) => Some(choice.key.to_ascii_lowercase()),
                _ => None,
            })
            .chain(std::iter::once(help_key))
            .collect();

        let transform = self.transform.take();

        let ans = ui::Input::new(
            ExpandPrompt {
                prompt: widgets::Prompt::new(&*message).with_hint(&hint),
                input: widgets::CharInput::new(|c| {
                    let c = c.to_ascii_lowercase();
                    hint.chars()
                        .find(|o| o.eq_ignore_ascii_case(&c))
                        .and(Some(c))
                }),
                select: widgets::Select::new(self),
                expanded: false,
            },
            b,
        )
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

        Ok(Answer::ExpandItem(ans))
    }
}

pub struct ExpandBuilder<'a> {
    opts: Options<'a>,
    expand: Expand<'a>,
    keys: HashSet<char>,
}

impl<'a> ExpandBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        ExpandBuilder {
            opts: Options::new(name),
            expand: Default::default(),
            keys: HashSet::default(),
        }
    }

    pub fn default(mut self, default: char) -> Self {
        self.expand.default = default;
        self
    }

    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.expand
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.expand.choices.choices.push(Choice::DefaultSeparator);
        self
    }

    pub fn choice<I: Into<String>>(mut self, mut key: char, name: I) -> Self {
        key = key.to_ascii_lowercase();
        if key == 'h' {
            panic!("Reserved key 'h'");
        }
        if self.keys.contains(&key) {
            panic!("Duplicate key '{}'", key);
        }

        self.keys.insert(key);

        self.expand.choices.choices.push(Choice::Choice(ExpandItem {
            key,
            name: Text::new(name.into()),
        }));

        self
    }

    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<ExpandItem<String>>>,
        I: IntoIterator<Item = T>,
    {
        let Self {
            ref mut keys,
            ref mut expand,
            ..
        } = self;

        expand
            .choices
            .choices
            .extend(choices.into_iter().map(|c| match c.into() {
                Choice::Choice(ExpandItem { name, mut key }) => {
                    key = key.to_ascii_lowercase();
                    if key == 'h' {
                        panic!("Reserved key 'h'");
                    }
                    if keys.contains(&key) {
                        panic!("Duplicate key '{}'", key);
                    }
                    keys.insert(key);

                    Choice::Choice(ExpandItem {
                        name: Text::new(name),
                        key,
                    })
                }
                Choice::Separator(s) => Choice::Separator(s),
                Choice::DefaultSeparator => Choice::DefaultSeparator,
            }));

        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.expand.choices.set_page_size(page_size);
        self
    }

    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.expand.choices.set_should_loop(should_loop);
        self
    }

    crate::impl_options_builder!();
    crate::impl_transform_builder!(ExpandItem<String>; expand);

    pub fn build(self) -> super::Question<'a> {
        super::Question::new(self.opts, super::QuestionKind::Expand(self.expand))
    }
}

impl<'a> From<ExpandBuilder<'a>> for super::Question<'a> {
    fn from(builder: ExpandBuilder<'a>) -> Self {
        builder.build()
    }
}

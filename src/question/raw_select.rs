use ui::{
    backend::Backend,
    error,
    events::KeyEvent,
    style::{Color, Stylize},
    widgets::{self, List, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Options, Transform};
use crate::{Answer, Answers, ListItem};

// Kind of a bad name
#[derive(Debug, Default)]
pub struct RawSelect<'a> {
    choices: super::ChoiceList<(usize, Text<String>)>,
    transform: Transform<'a, ListItem>,
}

struct RawSelectPrompt<'a> {
    prompt: widgets::Prompt<&'a str>,
    select: widgets::Select<RawSelect<'a>>,
    input: widgets::StringInput,
}

impl RawSelectPrompt<'_> {
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
                .1
                .text,
        }
    }
}

impl Prompt for RawSelectPrompt<'_> {
    type ValidateErr = &'static str;
    type Output = ListItem;

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.select.get_at() >= self.select.list.len() {
            Err("Please enter a valid choice")
        } else {
            Ok(Validation::Finish)
        }
    }

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

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl Widget for RawSelectPrompt<'_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.prompt.render(layout, b)?;
        self.select.render(layout, b)?;
        b.write_all(ANSWER_PROMPT)?;
        layout.line_offset += ANSWER_PROMPT.len() as u16;
        self.input.render(layout, b)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        // We don't need to add 1 for the answer prompt because this will over count by one
        let height = self.prompt.height(layout) + self.select.height(layout);
        layout.offset_y += 1;
        height
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.input.handle_key(key) {
            if let Ok(n) = self.input.value().parse::<usize>() {
                if n <= self.select.list.len() && n > 0 {
                    let pos = self.select.list.choices.choices[(n-1)..].iter().position(
                        |choice| matches!(choice, Choice::Choice((i, _)) if *i == n),
                    );

                    if let Some(pos) = pos {
                        self.select.set_at(pos + n - 1);
                        return true;
                    }
                }
            }

            self.select.set_at(self.select.list.len() + 1);
            true
        } else if self.select.handle_key(key) {
            let at = self.select.get_at();
            let index = self.select.list.choices[at].as_ref().unwrap_choice().0;
            self.input.set_value(index.to_string());
            true
        } else {
            false
        }
    }

    fn cursor_pos(&mut self, mut layout: ui::layout::Layout) -> (u16, u16) {
        let w = self
            .input
            .cursor_pos(layout.with_line_offset(ANSWER_PROMPT.len() as u16))
            .0;
        (w, self.height(&mut layout) - 1)
    }
}

impl widgets::List for RawSelect<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        match &mut self.choices[index] {
            &mut Choice::Choice((index, ref mut name)) => {
                if hovered {
                    b.set_fg(Color::Cyan)?;
                }

                write!(b, "  {}) ", index)?;

                layout.offset_x += (index as f64).log10() as u16 + 5;
                name.render(&mut layout, b)?;

                if hovered {
                    b.set_fg(Color::Reset)?;
                }
            }
            separator => {
                b.set_fg(Color::DarkGrey)?;
                b.write_all(b"   ")?;
                super::get_sep_str(separator)
                    .render(&mut layout.with_line_offset(3), b)?;
                b.set_fg(Color::Reset)?;
            }
        }

        Ok(())
    }

    fn is_selectable(&self, index: usize) -> bool {
        !self.choices[index].is_separator()
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        match self.choices[index] {
            Choice::Choice((index, ref mut c)) => {
                layout.offset_x += (index as f64).log10() as u16 + 5;
                c.height(&mut layout)
            }
            _ => 1,
        }
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

impl RawSelect<'_> {
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
            RawSelectPrompt {
                input: widgets::StringInput::new(|c| {
                    if c.is_digit(10) {
                        Some(c)
                    } else {
                        None
                    }
                }),
                select,
                prompt: widgets::Prompt::new(&message),
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

        Ok(Answer::ListItem(ans))
    }
}

pub struct RawSelectBuilder<'a> {
    opts: Options<'a>,
    list: RawSelect<'a>,
    choice_count: usize,
}

impl<'a> RawSelectBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        RawSelectBuilder {
            opts: Options::new(name),
            list: Default::default(),
            // It is one indexed for the user
            choice_count: 1,
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
            .push(Choice::Separator(text.into()));
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.list.choices.choices.push(Choice::DefaultSeparator);
        self
    }

    pub fn choice<I: Into<String>>(mut self, choice: I) -> Self {
        self.list.choices.choices.push(Choice::Choice((
            self.choice_count,
            Text::new(choice.into()),
        )));
        self.choice_count += 1;
        self
    }

    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<String>>,
        I: IntoIterator<Item = T>,
    {
        let choice_count = &mut self.choice_count;
        self.list
            .choices
            .choices
            .extend(choices.into_iter().map(|choice| match choice.into() {
                Choice::Choice(c) => {
                    let choice = Choice::Choice((*choice_count, Text::new(c)));
                    *choice_count += 1;
                    choice
                }
                Choice::Separator(s) => Choice::Separator(s),
                Choice::DefaultSeparator => Choice::DefaultSeparator,
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
        super::Question::new(self.opts, super::QuestionKind::RawSelect(self.list))
    }
}

impl<'a> From<RawSelectBuilder<'a>> for super::Question<'a> {
    fn from(builder: RawSelectBuilder<'a>) -> Self {
        builder.build()
    }
}

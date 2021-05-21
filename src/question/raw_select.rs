use ui::{
    backend::{Backend, Color, Stylize},
    error,
    events::KeyEvent,
    widgets::{self, List, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Options, Transform};
use crate::{Answer, Answers, ListItem};

// Kind of a bad name
#[derive(Debug, Default)]
pub struct RawSelect<'t> {
    choices: super::ChoiceList<(usize, Text<String>)>,
    transform: Transform<'t, ListItem>,
}

struct RawSelectPrompt<'t> {
    message: String,
    list: widgets::ListPicker<RawSelect<'t>>,
    input: widgets::StringInput,
}

impl RawSelectPrompt<'_> {
    fn finish_index(self, index: usize) -> ListItem {
        ListItem {
            index,
            name: self
                .list
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

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        if self.list.get_at() >= self.list.list.len() {
            Err("Please enter a valid index")
        } else {
            Ok(Validation::Finish)
        }
    }

    fn finish(self) -> Self::Output {
        let index = self.list.get_at();
        self.finish_index(index)
    }

    fn has_default(&self) -> bool {
        self.list.list.choices.default().is_some()
    }

    fn finish_default(self) -> Self::Output {
        let index = self.list.list.choices.default().unwrap();
        self.finish_index(index)
    }
}

crate::cfg_async! {
#[async_trait::async_trait]
impl ui::AsyncPrompt for RawSelectPrompt<'_> {
    async fn finish_async(self) -> Self::Output {
        self.finish()
    }

    fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
        Some(self.validate())
    }
}
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl Widget for RawSelectPrompt<'_> {
    fn render<B: Backend>(
        &mut self,
        mut layout: ui::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.list.render(layout, b)?;
        b.write_all(ANSWER_PROMPT)?;
        layout.line_offset += ANSWER_PROMPT.len() as u16;
        self.input.render(layout, b)
    }

    fn height(&mut self, layout: ui::Layout) -> u16 {
        self.list.height(layout) + 1
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.input.handle_key(key) {
            if let Ok(n) = self.input.value().parse::<usize>() {
                if n < self.list.list.len() && n > 0 {
                    let pos = self.list.list.choices.choices[(n-1)..].iter().position(
                        |choice| matches!(choice, Choice::Choice((i, _)) if *i == n),
                    );

                    if let Some(pos) = pos {
                        self.list.set_at(pos + n - 1);
                        return true;
                    }
                }
            }

            self.list.set_at(self.list.list.len() + 1);
            true
        } else if self.list.handle_key(key) {
            let at = self.list.get_at();
            let index = self.list.list.choices[at].as_ref().unwrap_choice().0;
            self.input.set_value(index.to_string());
            true
        } else {
            false
        }
    }

    fn cursor_pos(&mut self, layout: ui::Layout) -> (u16, u16) {
        let w = self
            .input
            .cursor_pos(layout.with_line_offset(ANSWER_PROMPT.len() as u16))
            .0;
        (w, self.height(layout) as u16)
    }
}

impl widgets::List for RawSelect<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::Layout,
        b: &mut B,
    ) -> error::Result<()> {
        match &mut self.choices[index] {
            &mut Choice::Choice((index, ref mut name)) => {
                if hovered {
                    b.set_fg(Color::Cyan)?;
                }

                write!(b, "  {}) ", index)?;

                layout.offset_x += (index as f64).log10() as u16 + 5;
                name.render(layout, b)?;

                if hovered {
                    b.set_fg(Color::Reset)?;
                }
            }
            separator => {
                b.set_fg(Color::DarkGrey)?;
                b.write_all(b"   ")?;
                super::get_sep_str(separator)
                    .render(layout.with_line_offset(3), b)?;
                b.set_fg(Color::Reset)?;
            }
        }

        Ok(())
    }

    fn is_selectable(&self, index: usize) -> bool {
        !self.choices[index].is_separator()
    }

    fn height_at(&mut self, index: usize, layout: ui::Layout) -> u16 {
        match self.choices[index] {
            Choice::Choice(ref mut c) => c.1.height(layout),
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

        let mut list = widgets::ListPicker::new(self);
        if let Some(default) = list.list.choices.default() {
            list.set_at(default);
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
                list,
                message,
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(ans.name.lines().next().unwrap().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::ListItem(ans))
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

        let mut list = widgets::ListPicker::new(self);
        if let Some(default) = list.list.choices.default() {
            list.set_at(default);
        }

        let ans = ui::Input::new(RawSelectPrompt {
            input: widgets::StringInput::new(|c| {
                if c.is_digit(10) {
                    Some(c)
                } else {
                    None
                }
            }),
            list,
            message,
        }, b)
        .run_async(events)
        .await?;

        match transform {
            Transform::Async(transform) => transform(&ans, answers, b).await?,
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(ans.name.lines().next().unwrap().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?
            }
        }

        Ok(Answer::ListItem(ans))
    }
    }
}

pub struct RawSelectBuilder<'m, 'w, 't> {
    opts: Options<'m, 'w>,
    list: RawSelect<'t>,
    choice_count: usize,
}

impl<'m, 'w, 't> RawSelectBuilder<'m, 'w, 't> {
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

    pub fn build(self) -> super::Question<'m, 'w, 'static, 'static, 't> {
        super::Question::new(self.opts, super::QuestionKind::RawSelect(self.list))
    }
}

impl<'m, 'w, 't> From<RawSelectBuilder<'m, 'w, 't>>
    for super::Question<'m, 'w, 'static, 'static, 't>
{
    fn from(builder: RawSelectBuilder<'m, 'w, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(RawSelectBuilder<'t>; (this, opts) => {
    RawSelectBuilder {
        opts,
        list: this.list,
        choice_count: this.choice_count,
    }
});

crate::impl_transform_builder!(RawSelectBuilder<'m, 'w, t> ListItem; (this, transform) => {
    RawSelectBuilder {
        opts: this.opts,
        choice_count: this.choice_count,
        list: RawSelect {
            transform,
            choices: this.list.choices,
        }
    }
});

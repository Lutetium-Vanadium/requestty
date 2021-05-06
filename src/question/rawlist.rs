use ui::{
    backend::{Backend, Color, Stylize},
    error,
    events::KeyEvent,
    widgets::{self, List},
    Prompt, Validation, Widget,
};

use super::{Choice, Options, Transform};
use crate::{Answer, Answers, ListItem};

#[derive(Debug, Default)]
pub struct Rawlist<'t> {
    choices: super::ChoiceList<(usize, String)>,
    transform: Transform<'t, ListItem>,
}

struct RawlistPrompt<'t> {
    message: String,
    list: widgets::ListPicker<Rawlist<'t>>,
    input: widgets::StringInput,
}

impl RawlistPrompt<'_> {
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
                .1,
        }
    }
}

impl Prompt for RawlistPrompt<'_> {
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
impl ui::AsyncPrompt for RawlistPrompt<'_> {
    async fn finish_async(self) -> Self::Output {
        self.finish()
    }

    fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
        Some(self.validate())
    }
}
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl Widget for RawlistPrompt<'_> {
    fn render<B: Backend>(&mut self, _: usize, b: &mut B) -> error::Result<()> {
        let max_width = b.size()?.width as usize;
        self.list.render(max_width, b)?;
        b.write_all(ANSWER_PROMPT)?;
        self.input.render(max_width - ANSWER_PROMPT.len(), b)
    }

    fn height(&self) -> usize {
        self.list.height() + 1
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

    fn cursor_pos(&self, _: u16) -> (u16, u16) {
        let w = self.input.cursor_pos(ANSWER_PROMPT.len() as u16).0;
        (w, self.height() as u16)
    }
}

impl widgets::List for Rawlist<'_> {
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        max_width: usize,
        b: &mut B,
    ) -> error::Result<()> {
        match &self.choices[index] {
            Choice::Choice((index, name)) => {
                if hovered {
                    b.set_fg(Color::Cyan)?;
                }

                write!(b, "  {}) ", index)?;
                name.as_str()
                    .render(max_width - (*index as f64).log10() as usize + 5, b)?;

                if hovered {
                    b.set_fg(Color::Reset)?;
                }
            }
            separator => {
                b.set_fg(Color::DarkGrey)?;
                b.write_all(b"   ")?;
                super::get_sep_str(separator).render(max_width - 3, b)?;
                b.set_fg(Color::Reset)?;
            }
        }

        Ok(())
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

impl Rawlist<'_> {
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
            RawlistPrompt {
                input: widgets::StringInput::new(|c| c.is_digit(10).then(|| c)),
                list,
                message,
            },
            b,
        )
        .run(events)?;

        match transform {
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(ans.name.as_str().cyan())?;
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

        let ans = ui::Input::new(RawlistPrompt {
            input: widgets::StringInput::new(|c| c.is_digit(10).then(|| c)),
            list,
            message,
        }, b)
        .run_async(events)
        .await?;

        match transform {
            Transform::Async(transform) => transform(&ans, answers, b).await?,
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(ans.name.as_str().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?
            }
        }

        Ok(Answer::ListItem(ans))
    }
    }
}

pub struct RawlistBuilder<'m, 'w, 't> {
    opts: Options<'m, 'w>,
    list: Rawlist<'t>,
    choice_count: usize,
}

impl<'m, 'w, 't> RawlistBuilder<'m, 'w, 't> {
    pub(crate) fn new(name: String) -> Self {
        RawlistBuilder {
            opts: Options::new(name.into()),
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
        self.list
            .choices
            .choices
            .push(Choice::Choice((self.choice_count, choice.into())));
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
                    let choice = Choice::Choice((*choice_count, c));
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
        super::Question::new(self.opts, super::QuestionKind::Rawlist(self.list))
    }
}

impl<'m, 'w, 't> From<RawlistBuilder<'m, 'w, 't>>
    for super::Question<'m, 'w, 'static, 'static, 't>
{
    fn from(builder: RawlistBuilder<'m, 'w, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(RawlistBuilder<'t>; (this, opts) => {
    RawlistBuilder {
        opts,
        list: this.list,
        choice_count: this.choice_count,
    }
});

crate::impl_transform_builder!(RawlistBuilder<'m, 'w, t> ListItem; (this, transform) => {
    RawlistBuilder {
        opts: this.opts,
        choice_count: this.choice_count,
        list: Rawlist {
            transform,
            choices: this.list.choices,
        }
    }
});

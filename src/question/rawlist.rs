use crossterm::{
    event, queue,
    style::{Color, Colorize, ResetColor, SetForegroundColor},
    terminal,
};
use ui::{widgets, Prompt, Validation, Widget};
use widgets::List;

use crate::{
    answer::{Answer, ListItem},
    error, Answers,
};

use super::{Choice, Options, Transformer};

#[derive(Debug, Default)]
pub struct Rawlist<'t> {
    choices: super::ChoiceList<(usize, String)>,
    transformer: Transformer<'t, ListItem>,
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
    fn render<W: std::io::Write>(&mut self, _: usize, w: &mut W) -> crossterm::Result<()> {
        let max_width = terminal::size()?.0 as usize;
        self.list.render(max_width, w)?;
        w.write_all(ANSWER_PROMPT)?;
        self.input.render(max_width - ANSWER_PROMPT.len(), w)
    }

    fn height(&self) -> usize {
        self.list.height() + 1
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        if self.input.handle_key(key) {
            if let Ok(mut n) = self.input.value().parse::<usize>() {
                if n < self.list.list.len() && n > 0 {
                    // Choices are 1 indexed for the user
                    n -= 1;

                    let pos = self.list.list.choices.choices[n..]
                        .iter()
                        .position(|choice| matches!(choice, Choice::Choice((i, _)) if *i == n));

                    if let Some(pos) = pos {
                        self.list.set_at(pos + n);
                        return true;
                    }
                }
            }

            self.list.set_at(self.list.list.len() + 1);
            true
        } else if self.list.handle_key(key) {
            let at = self.list.get_at();
            let index = self.list.list.choices[at].as_ref().unwrap_choice().0 + 1;
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
    fn render_item<W: std::io::Write>(
        &mut self,
        index: usize,
        hovered: bool,
        max_width: usize,
        w: &mut W,
    ) -> crossterm::Result<()> {
        match &self.choices[index] {
            Choice::Choice((index, name)) => {
                if hovered {
                    queue!(w, SetForegroundColor(Color::DarkCyan))?;
                }

                write!(w, "  {}) ", index + 1)?;
                name.as_str()
                    .render(max_width - (*index as f64).log10() as usize + 5, w)?;

                if hovered {
                    queue!(w, ResetColor)?;
                }
            }
            Choice::Separator(s) => {
                queue!(w, SetForegroundColor(Color::DarkGrey))?;
                w.write_all(b"   ")?;
                super::get_sep_str(s).render(max_width - 3, w)?;
                queue!(w, ResetColor)?;
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
    pub(crate) fn ask<W: std::io::Write>(
        mut self,
        message: String,
        answers: &Answers,
        w: &mut W,
    ) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let mut list = widgets::ListPicker::new(self);
        if let Some(default) = list.list.choices.default() {
            list.set_at(default);
        }

        let ans = ui::Input::new(RawlistPrompt {
            input: widgets::StringInput::new(|c| c.is_digit(10).then(|| c)),
            list,
            message,
        })
        .run(w)?;

        match transformer {
            Transformer::Sync(transformer) => transformer(&ans, answers, w)?,
            _ => writeln!(w, "{}", ans.name.as_str().dark_cyan())?,
        }

        Ok(Answer::ListItem(ans))
    }

    crate::cfg_async! {
    pub(crate) async fn ask_async<W: std::io::Write>(
        mut self,
        message: String,
        answers: &Answers,
        w: &mut W,
    ) -> error::Result<Answer> {
        let transformer = self.transformer.take();

        let mut list = widgets::ListPicker::new(self);
        if let Some(default) = list.list.choices.default() {
            list.set_at(default);
        }

        let ans = ui::AsyncInput::new(RawlistPrompt {
            input: widgets::StringInput::new(|c| c.is_digit(10).then(|| c)),
            list,
            message,
        })
        .run(w)
        .await?;

        match transformer {
            Transformer::Async(transformer) => transformer(&ans, answers, w).await?,
            Transformer::Sync(transformer) => transformer(&ans, answers, w)?,
            _ => writeln!(w, "{}", ans.name.as_str().dark_cyan())?,
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
    pub fn default(mut self, default: usize) -> Self {
        self.list.choices.set_default(default);
        self
    }

    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.list
            .choices
            .choices
            .push(Choice::Separator(Some(text.into())));
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.list.choices.choices.push(Choice::Separator(None));
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

crate::impl_transformer_builder!(RawlistBuilder<'m, 'w, t> ListItem; (this, transformer) => {
    RawlistBuilder {
        opts: this.opts,
        choice_count: this.choice_count,
        list: Rawlist {
            transformer,
            choices: this.list.choices,
        }
    }
});

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn rawlist<N: Into<String>>(name: N) -> RawlistBuilder<'static, 'static, 'static> {
        RawlistBuilder {
            opts: Options::new(name.into()),
            list: Default::default(),
            choice_count: 0,
        }
    }
}

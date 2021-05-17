use ui::{
    backend::{Backend, Color, Stylize},
    error,
    events::KeyEvent,
    widgets, Prompt, Widget,
};

use super::{Options, Transform};
use crate::{Answer, Answers, ListItem};

#[derive(Debug, Default)]
pub struct List<'t> {
    choices: super::ChoiceList<String>,
    transform: Transform<'t, ListItem>,
}

struct ListPrompt<'t> {
    message: String,
    picker: widgets::ListPicker<List<'t>>,
}

impl ListPrompt<'_> {
    fn finish_index(self, index: usize) -> ListItem {
        ListItem {
            index,
            name: self
                .picker
                .finish()
                .choices
                .choices
                .swap_remove(index)
                .unwrap_choice(),
        }
    }
}

impl Prompt for ListPrompt<'_> {
    type ValidateErr = &'static str;
    type Output = ListItem;

    fn prompt(&self) -> &str {
        &self.message
    }

    fn hint(&self) -> Option<&str> {
        Some("(Use arrow keys)")
    }

    fn finish(self) -> Self::Output {
        let index = self.picker.get_at();
        self.finish_index(index)
    }

    fn has_default(&self) -> bool {
        self.picker.list.choices.default().is_some()
    }
    fn finish_default(self) -> Self::Output {
        let index = self.picker.list.choices.default().unwrap();
        self.finish_index(index)
    }
}

crate::cfg_async! {
#[async_trait::async_trait]
impl ui::AsyncPrompt for ListPrompt<'_> {
    async fn finish_async(self) -> Self::Output {
        self.finish()
    }

    fn try_validate_sync(&mut self) -> Option<Result<ui::Validation, Self::ValidateErr>> {
        Some(self.validate())
    }
}
}

impl Widget for ListPrompt<'_> {
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
        self.picker.handle_key(key)
    }

    fn cursor_pos(&mut self, layout: ui::Layout) -> (u16, u16) {
        self.picker.cursor_pos(layout)
    }
}

impl widgets::List for List<'_> {
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

            if !self.is_selectable(index) {
                b.set_fg(Color::DarkGrey)?;
            }
        }

        layout.line_offset += 2;
        self.choices[index].as_str().render(layout, b)?;

        b.set_fg(Color::Reset)
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

impl List<'_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
    ) -> error::Result<Answer> {
        let transform = self.transform.take();
        let mut picker = widgets::ListPicker::new(self);
        if let Some(default) = picker.list.choices.default() {
            picker.set_at(default);
        }
        let ans = ui::Input::new(ListPrompt { message, picker }, b)
            .hide_cursor()
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
        let mut picker = widgets::ListPicker::new(self);
        if let Some(default) = picker.list.choices.default() {
            picker.set_at(default);
        }
        let ans = ui::Input::new(ListPrompt { message, picker }, b)
            .hide_cursor()
            .run_async(events)
            .await?;

        match transform {
            Transform::Async(transform) => transform(&ans, answers, b).await?,
            Transform::Sync(transform) => transform(&ans, answers, b)?,
            _ => {
                b.write_styled(ans.name.as_str().cyan())?;
                b.write_all(b"\n")?;
                b.flush()?;
            }
        }

        Ok(Answer::ListItem(ans))
    }
    }
}

pub struct ListBuilder<'m, 'w, 't> {
    opts: Options<'m, 'w>,
    list: List<'t>,
}

impl<'m, 'w, 't> ListBuilder<'m, 'w, 't> {
    pub(crate) fn new(name: String) -> Self {
        ListBuilder {
            opts: Options::new(name),
            list: Default::default(),
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
            .push(super::Choice::Separator(text.into()));
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::DefaultSeparator);
        self
    }

    pub fn choice<I: Into<String>>(mut self, choice: I) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::Choice(choice.into()));
        self
    }

    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<super::Choice<String>>,
        I: IntoIterator<Item = T>,
    {
        self.list
            .choices
            .choices
            .extend(choices.into_iter().map(Into::into));
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
        super::Question::new(self.opts, super::QuestionKind::List(self.list))
    }
}

impl<'m, 'w, 't> From<ListBuilder<'m, 'w, 't>>
    for super::Question<'m, 'w, 'static, 'static, 't>
{
    fn from(builder: ListBuilder<'m, 'w, 't>) -> Self {
        builder.build()
    }
}

crate::impl_options_builder!(ListBuilder<'t>; (this, opts) => {
    ListBuilder {
        opts,
        list: this.list,
    }
});

crate::impl_transform_builder!(ListBuilder<'m, 'w, t> ListItem; (this, transform) => {
    ListBuilder {
        opts: this.opts,
        list: List {
            transform,
            choices: this.list.choices,
        }
    }
});

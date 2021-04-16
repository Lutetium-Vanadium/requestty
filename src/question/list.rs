use std::fmt;

use crossterm::{
    queue,
    style::{Color, Colorize, Print, ResetColor, SetForegroundColor},
};
use ui::{widgets, Widget};

use crate::{
    answer::{Answer, ListItem},
    error,
};

use super::{none, some, Options, Transformer};

#[derive(Default)]
pub struct List<'t> {
    choices: super::ChoiceList<String>,
    transformer: Option<Box<Transformer<'t, ListItem>>>,
}

impl fmt::Debug for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("List")
            .field("choices", &self.choices)
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
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

impl ui::Prompt for ListPrompt<'_> {
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

impl Widget for ListPrompt<'_> {
    fn render<W: std::io::Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        self.picker.render(max_width, w)
    }

    fn height(&self) -> usize {
        self.picker.height()
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.picker.handle_key(key)
    }

    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        self.picker.cursor_pos(prompt_len)
    }
}

impl widgets::List for List<'_> {
    fn render_item<W: std::io::Write>(
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

            if !self.is_selectable(index) {
                queue!(w, SetForegroundColor(Color::DarkGrey))?;
            }
        }

        self.choices[index].as_str().render(max_width - 2, w)?;

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

impl List<'_> {
    pub fn ask<W: std::io::Write>(mut self, message: String, w: &mut W) -> error::Result<Answer> {
        let transformer = self.transformer.take();
        let mut picker = widgets::ListPicker::new(self);
        if let Some(default) = picker.list.choices.default() {
            picker.set_at(default);
        }
        let ans = ui::Input::new(ListPrompt { picker, message })
            .hide_cursor()
            .run(w)?;

        match transformer {
            Some(transformer) => transformer(&ans, w)?,
            None => writeln!(w, "{}", ans.name.as_str().dark_cyan())?,
        }

        Ok(Answer::ListItem(ans))
    }
}

pub struct ListBuilder<'m, 'w, 't> {
    opts: Options<'m, 'w>,
    list: List<'t>,
}

impl<'m, 'w, 't> ListBuilder<'m, 'w, 't> {
    pub fn default(mut self, default: usize) -> Self {
        self.list.choices.set_default(default);
        self
    }

    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::Separator(Some(text.into())));
        self
    }

    pub fn default_separator(mut self) -> Self {
        self.list
            .choices
            .choices
            .push(super::Choice::Separator(None));
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

impl<'m, 'w, 't> From<ListBuilder<'m, 'w, 't>> for super::Question<'m, 'w, 'static, 'static, 't> {
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

crate::impl_transformer_builder!(ListBuilder<'m, 'w, t> ListItem; (this, transformer) => {
    ListBuilder {
        opts: this.opts,
        list: List {
            transformer,
            choices: this.list.choices,
        }
    }
});

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn list<N: Into<String>>(name: N) -> ListBuilder<'static, 'static, 'static> {
        ListBuilder {
            opts: Options::new(name.into()),
            list: Default::default(),
        }
    }
}

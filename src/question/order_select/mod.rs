mod builder;

use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyEvent},
    style::Color,
    widgets::{self, Text},
    Prompt, Widget,
};

use crate::{Answer, Answers, ListItem};

use super::{
    handler::{Filter, Transform, Validate}, choice::SelectList,
};

pub use builder::OrderSelectBuilder;

#[cfg(test)]
mod tests;

// =============================================================================
//
// =============================================================================

#[derive(Debug)]
pub(super) struct OrderSelect<'a> {
    choices: SelectList<OrderSelectItem>,
    max_index_width: usize,
    moving: bool,

    transform: Transform<'a, [OrderSelectItem]>,
    validate: Validate<'a, [OrderSelectItem]>,
    filter: Filter<'a, Vec<OrderSelectItem>>,
}

impl<'a> Default for OrderSelect<'a> {
    fn default() -> Self {
        Self { 
            choices: SelectList::new(|_| true),

            // can't put
            // `..Default::default()`
            // because of recursion
            max_index_width: Default::default(),
            moving: Default::default(),
            transform: Default::default(),
            validate: Default::default(),
            filter: Default::default(),
        }
    }
} 

impl widgets::List for OrderSelect<'_> {
    fn render_item<B: ui::backend::Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> std::io::Result<()> {
        let symbol_set = ui::symbols::current();

        if hovered {
            if self.moving {
                b.set_bg(Color::Cyan)?;
                b.set_fg(Color::Black)?;
            } else {
                b.set_fg(Color::Cyan)?;
            }

            write!(b, "{} ", symbol_set.pointer)?;
        } else {
            b.write_all(b"  ")?;
        }

        write!(
            b,
            "{:>width$}. ",
            index,
            width = self.max_index_width as usize
        )?;

        layout.offset_x += self.max_index_width as u16 + 4;

        self.choices[index].render(&mut layout, b)?;

        b.set_fg(Color::Reset)?;
        b.set_bg(Color::Reset)
    }

    fn is_selectable(&self, _index: usize) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        self.choices.page_size()
    }

    fn should_loop(&self) -> bool {
        self.choices.should_loop()
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        layout.offset_x += self.max_index_width as u16 + 4;
        self.choices[index].height(&mut layout)
    }

    fn len(&self) -> usize {
        self.choices.len()
    }
}

impl<'c> OrderSelect<'c> {
    fn into_order_select_prompt<'a>(
        self,
        message: &'a str,
        answers: &'a Answers,
    ) -> OrderSelectPrompt<'a, 'c> {
        OrderSelectPrompt {
            prompt: widgets::Prompt::new(message).with_hint(
                "Press <space> to take and place an option",
            ),
            select: widgets::Select::new(self),
            answers,
        }
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        on_esc: ui::OnEsc,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Option<Answer>> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_order_select_prompt(&message, answers), b)
            .hide_cursor()
            .on_esc(on_esc)
            .run(events)?;

        crate::write_final!(transform, message, ans [ref], answers, b, |ans| {
            b.set_fg(Color::Cyan)?;
            print_comma_separated(
                ans.iter().map(|item| {
                    item.text()
                }),
                b,
            )?;
            b.set_fg(Color::Reset)?;
        })
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

// =============================================================================
//
// =============================================================================

struct OrderSelectPrompt<'a, 'c> {
    prompt: widgets::Prompt<&'a str>,
    select: widgets::Select<OrderSelect<'c>>,
    answers: &'a Answers,
}

impl Prompt for OrderSelectPrompt<'_, '_> {
    type ValidateErr = widgets::Text<String>;
    type Output = Vec<OrderSelectItem>;

    fn finish(self) -> Self::Output {
        let OrderSelect {
            choices,
            filter,
            ..
        } = self.select.into_inner();

        let mut c = choices.choices;

        if let Filter::Sync(filter) = filter {
            c = filter(c, self.answers);
        }

        c
    }

    fn validate(&mut self) -> Result<ui::Validation, Self::ValidateErr> {
        if let Validate::Sync(ref mut validate) = self.select.list.validate {
            validate(&self.select.list.choices.choices, self.answers)?;
        }
        Ok(ui::Validation::Finish)
    }
}

impl Widget for OrderSelectPrompt<'_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        backend: &mut B,
    ) -> io::Result<()> {
        self.prompt.render(layout, backend)?;
        self.select.render(layout, backend)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.select.height(layout) - 1
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.select.cursor_pos(layout)
    }

    fn handle_key(&mut self, key: ui::events::KeyEvent) -> bool {
        let prev_at = self.select.get_at();
    
        if let ui::events::KeyCode::Char(' ') = key.code {
            self.select.list.moving = !self.select.list.moving;
        } else if self.select.handle_key(key) {
            if self.select.list.moving {
                let new_at = self.select.get_at();
    
                if prev_at < new_at {
                    self.select.list.choices.choices[prev_at..=new_at].rotate_left(1);
                } else {
                    self.select.list.choices.choices[new_at..=prev_at].rotate_right(1);
                }
            }
        } else {
            return false;
        }
    
        true
    }
}

// =============================================================================
// 
// =============================================================================

/// The representation of each choice in an [`OrderSelect`].
///
/// It is different from [`ListItem`](crate::answer::ListItem) due to an implementation detail.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderSelectItem {
    index: usize,
    text: Text<String>,
}

impl OrderSelectItem {
    /// The index of the choice
    pub fn index(&self) -> usize {
        self.index
    }

    /// The content of the choice -- it is what is displayed to the user
    pub fn text(&self) -> &str {
        &self.text.text
    }
}

impl Widget for OrderSelectItem {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        backend: &mut B,
    ) -> io::Result<()> {
        self.text.render(layout, backend)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.text.height(layout)
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.text.cursor_pos(layout)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.text.handle_key(key)
    }
}

impl Into<ListItem> for OrderSelectItem {
    fn into(self) -> ListItem {
        ListItem { index: self.index, text: self.text.text }
    }
}

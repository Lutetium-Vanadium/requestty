use std::io;

use ui::{
    backend::Backend,
    events::{EventIterator, KeyEvent},
    style::{Color, Stylize},
    widgets::{self, List, Text},
    Prompt, Validation, Widget,
};

use super::{Choice, Transform};
use crate::{Answer, Answers, ListItem};

pub use builder::RawSelectBuilder;

mod builder;

#[cfg(test)]
mod tests;

// Kind of a bad name
#[derive(Debug, Default)]
pub(super) struct RawSelect<'a> {
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
            text: self
                .select
                .into_inner()
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
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl Widget for RawSelectPrompt<'_> {
    fn render<B: Backend>(&mut self, layout: &mut ui::layout::Layout, b: &mut B) -> io::Result<()> {
        self.prompt.render(layout, b)?;
        self.select.render(layout, b)?;
        b.write_all(ANSWER_PROMPT)?;
        layout.line_offset += ANSWER_PROMPT.len() as u16;
        self.input.render(layout, b)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        // We don't need to add 1 for the answer prompt because this will over count by one
        let height = self.prompt.height(layout) + self.select.height(layout);
        layout.line_offset = ANSWER_PROMPT.len() as u16;
        height + self.input.height(layout) - 1
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.input.handle_key(key) {
            if let Ok(n) = self.input.value().parse::<usize>() {
                if n <= self.select.list.len() && n > 0 {
                    let pos = self.select.list.choices.choices[(n - 1)..]
                        .iter()
                        .position(|choice| matches!(choice, Choice::Choice((i, _)) if *i == n));

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
    ) -> io::Result<()> {
        match &mut self.choices[index] {
            &mut Choice::Choice((index, ref mut text)) => {
                if hovered {
                    b.set_fg(Color::Cyan)?;
                }

                write!(b, "  {}) ", index)?;

                layout.offset_x += (index as f64).log10() as u16 + 5;
                text.render(&mut layout, b)?;

                if hovered {
                    b.set_fg(Color::Reset)?;
                }
            }
            separator => {
                b.set_fg(Color::DarkGrey)?;
                b.write_all(b"   ")?;
                super::get_sep_str(separator).render(&mut layout.with_line_offset(3), b)?;
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

impl<'a> RawSelect<'a> {
    fn into_prompt(self, message: &'a str) -> RawSelectPrompt<'a> {
        let mut select = widgets::Select::new(self);
        if let Some(default) = select.list.choices.default() {
            select.set_at(default);
        }

        RawSelectPrompt {
            input: widgets::StringInput::with_filter_map(|c| {
                if c.is_digit(10) {
                    Some(c)
                } else {
                    None
                }
            }),
            select,
            prompt: widgets::Prompt::new(&message),
        }
    }

    pub(crate) fn ask<B: Backend, E: EventIterator>(
        mut self,
        message: String,
        answers: &Answers,
        b: &mut B,
        events: &mut E,
    ) -> ui::Result<Answer> {
        let transform = self.transform.take();

        let ans = ui::Input::new(self.into_prompt(&message), b).run(events)?;

        crate::write_final!(
            transform,
            message,
            &ans,
            answers,
            b,
            b.write_styled(
                &ans.text
                    .lines()
                    .next()
                    .expect("There must be at least one line in a `str`")
                    .cyan()
            )?
        );

        Ok(Answer::ListItem(ans))
    }
}

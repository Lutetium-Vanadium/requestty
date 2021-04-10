use std::io;

use crossterm::{
    event, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use ui::{widgets, Widget};

use crate::{answer::ListItem, error, Answer};

use super::Options;

pub struct Checkbox {
    // FIXME: What is type here?
    choices: super::ChoiceList<String>,
    selected: Vec<bool>,
}

struct CheckboxPrompt {
    picker: widgets::ListPicker<Checkbox>,
    opts: Options,
}

impl ui::Prompt for CheckboxPrompt {
    type ValidateErr = &'static str;
    type Output = Checkbox;

    fn prompt(&self) -> &str {
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        Some("(Press <space> to select, <a> to toggle all, <i> to invert selection)")
    }

    fn finish(self) -> Self::Output {
        self.picker.finish()
    }

    fn finish_default(self) -> Self::Output {
        unreachable!()
    }
    fn has_default(&self) -> bool {
        false
    }
}

impl Widget for CheckboxPrompt {
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

impl widgets::List for Checkbox {
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
}

impl Checkbox {
    pub fn ask<W: io::Write>(self, opts: super::Options, w: &mut W) -> error::Result<Answer> {
        // We cannot simply process the Vec<bool> to a HashSet<ListItem> since we want to print the
        // selected ones in order
        let checkbox = ui::Input::new(CheckboxPrompt {
            picker: widgets::ListPicker::new(self),
            opts,
        })
        .hide_cursor()
        .run(w)?;

        queue!(w, SetForegroundColor(Color::DarkCyan))?;
        print_comma_separated(
            checkbox
                .selected
                .iter()
                .zip(checkbox.choices.choices.iter())
                .filter_map(|item| match item {
                    (true, super::Choice::Choice(name)) => Some(name.as_str()),
                    _ => None,
                }),
            w,
        )?;

        w.write_all(b"\n")?;
        execute!(w, ResetColor)?;

        let ans = checkbox
            .selected
            .into_iter()
            .enumerate()
            .zip(checkbox.choices.choices.into_iter())
            .filter_map(|((index, is_selected), name)| match (is_selected, name) {
                (true, super::Choice::Choice(name)) => Some(ListItem { index, name }),
                _ => None,
            })
            .collect();

        Ok(Answer::ListItems(ans))
    }
}

impl super::Question {
    pub fn checkbox(name: String, message: String, choices: Vec<super::Choice<String>>) -> Self {
        Self::new(
            name,
            message,
            super::QuestionKind::Checkbox(Checkbox {
                selected: vec![false; choices.len()],
                choices: super::ChoiceList {
                    choices,
                    default: 0,
                    should_loop: true,
                    // FIXME: this should be something sensible. page size is currently not used so
                    // its fine for now
                    page_size: 0,
                },
            }),
        )
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

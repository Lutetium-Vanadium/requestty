use crossterm::{
    queue,
    style::{Color, Colorize, Print, ResetColor, SetForegroundColor},
};
use ui::{widgets, Widget};

use crate::{
    answer::{Answer, ListItem},
    error,
};

use super::Options;

pub struct List {
    // FIXME: Whats the correct type?
    choices: super::ChoiceList<String>,
}

struct ListPrompt {
    picker: widgets::ListPicker<List>,
    opts: Options,
}

impl ListPrompt {
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

impl ui::Prompt for ListPrompt {
    type ValidateErr = &'static str;
    type Output = ListItem;

    fn prompt(&self) -> &str {
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        Some("(Use arrow keys)")
    }

    fn finish(self) -> Self::Output {
        let index = self.picker.get_at();
        self.finish_index(index)
    }

    fn finish_default(self) -> Self::Output {
        let index = self.picker.list.choices.default;
        self.finish_index(index)
    }
}

impl Widget for ListPrompt {
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

impl widgets::List for List {
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
}

impl List {
    pub fn ask<W: std::io::Write>(self, opts: Options, w: &mut W) -> error::Result<Answer> {
        let ans = ui::Input::new(ListPrompt {
            picker: widgets::ListPicker::new(self),
            opts,
        })
        .hide_cursor()
        .run(w)?;

        writeln!(w, "{}", ans.name.as_str().dark_cyan())?;

        Ok(Answer::ListItem(ans))
    }
}

impl super::Question {
    pub fn list(
        name: String,
        message: String,
        choices: Vec<super::Choice<String>>,
        default: usize,
    ) -> Self {
        Self::new(
            name,
            message,
            super::QuestionKind::List(List {
                choices: super::ChoiceList {
                    choices,
                    default,
                    should_loop: true,
                    // FIXME: this should be something sensible. page size is currently not used so
                    // its fine for now
                    page_size: 0,
                },
            }),
        )
    }
}

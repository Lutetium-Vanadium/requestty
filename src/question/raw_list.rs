use crossterm::{
    event, queue,
    style::{Color, Colorize, ResetColor, SetForegroundColor},
    terminal,
};
use ui::{widgets, Validation, Widget};
use widgets::List;

use crate::{
    answer::{Answer, ListItem},
    error,
};

use super::{Choice, Options};

pub struct Rawlist {
    // FIXME: Whats the correct type?
    choices: super::ChoiceList<(usize, String)>,
}

struct RawlistPrompt {
    list: widgets::ListPicker<Rawlist>,
    input: widgets::StringInput,
    opts: Options,
}

impl RawlistPrompt {
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

impl ui::Prompt for RawlistPrompt {
    type ValidateErr = &'static str;
    type Output = ListItem;

    fn prompt(&self) -> &str {
        &self.opts.message
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

    fn finish_default(self) -> Self::Output {
        let index = self.list.list.choices.default;
        self.finish_index(index)
    }
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl Widget for RawlistPrompt {
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
            self.input.set_value(self.list.get_at().to_string());
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

impl widgets::List for Rawlist {
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
}

impl Rawlist {
    pub fn ask<W: std::io::Write>(self, opts: Options, w: &mut W) -> error::Result<Answer> {
        let ans = ui::Input::new(RawlistPrompt {
            input: widgets::StringInput::new(|c| c.is_digit(10).then(|| c)),
            list: widgets::ListPicker::new(self),
            opts,
        })
        .run(w)?;

        writeln!(w, "{}", ans.name.as_str().dark_cyan())?;

        Ok(Answer::ListItem(ans))
    }
}

impl super::Question {
    pub fn raw_list(
        name: String,
        message: String,
        choices: Vec<Choice<String>>,
        default: usize,
    ) -> Self {
        Self::new(
            name,
            message,
            super::QuestionKind::Rawlist(Rawlist {
                choices: super::ChoiceList {
                    choices: choices
                        .into_iter()
                        .scan(0, |index, choice| match choice {
                            Choice::Choice(s) => {
                                let res = Choice::Choice((*index, s));
                                *index += 1;
                                Some(res)
                            }
                            Choice::Separator(s) => Some(Choice::Separator(s)),
                        })
                        .collect(),
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

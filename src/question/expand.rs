use crossterm::{
    cursor, queue,
    style::{Color, Colorize, ResetColor, SetForegroundColor},
    terminal,
};
use ui::{widgets, Validation, Widget};

use crate::{answer::ExpandItem, error, Answer};

use super::{Choice, Options};

pub struct Expand {
    choices: super::ChoiceList<ExpandItem>,
    selected: Option<char>,
    default: Option<char>,
}

struct ExpandPrompt<'a, F> {
    list: widgets::ListPicker<Expand>,
    input: widgets::CharInput<F>,
    opts: Options,
    hint: &'a str,
    expanded: bool,
}

impl<F> ExpandPrompt<'_, F> {
    fn finish_with(self, c: char) -> ExpandItem {
        self.list
            .finish()
            .choices
            .choices
            .into_iter()
            .filter_map(|choice| match choice {
                Choice::Choice(choice) => Some(choice),
                _ => None,
            })
            .find(|item| item.key == c)
            .unwrap()
    }
}

impl<F: Fn(char) -> Option<char>> ui::Prompt for ExpandPrompt<'_, F> {
    type ValidateErr = &'static str;
    type Output = ExpandItem;

    fn prompt(&self) -> &str {
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        Some(self.hint)
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        match self.input.value() {
            Some('h') => {
                self.expanded = true;
                self.input.set_value(None);
                self.list.list.selected = None;
                Ok(Validation::Continue)
            }
            None if self.list.list.default.is_none() => Err("Please enter a command"),
            _ => Ok(Validation::Finish),
        }
    }

    fn finish(self) -> Self::Output {
        let c = self.input.value().unwrap();
        self.finish_with(c)
    }

    fn has_default(&self) -> bool {
        self.list.list.default.is_some()
    }

    fn finish_default(self) -> Self::Output {
        let c = self.list.list.default.unwrap();
        self.finish_with(c)
    }
}

const ANSWER_PROMPT: &[u8] = b"  Answer: ";

impl<F: Fn(char) -> Option<char>> ui::Widget for ExpandPrompt<'_, F> {
    fn render<W: std::io::Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        if self.expanded {
            let max_width = terminal::size()?.0 as usize - ANSWER_PROMPT.len();
            self.list.render(max_width, w)?;
            w.write_all(ANSWER_PROMPT)?;
            self.input.render(max_width, w)
        } else {
            self.input.render(max_width, w)?;

            if let Some(key) = self.input.value() {
                let name = &self
                    .list
                    .list
                    .choices
                    .choices
                    .iter()
                    .filter_map(|choice| match choice {
                        Choice::Choice(choice) => Some(choice),
                        _ => None,
                    })
                    .find(|item| item.key == key)
                    .map(|item| &*item.name)
                    .unwrap_or("Help, list all options");

                queue!(w, cursor::MoveToNextLine(1))?;

                write!(w, "{} {}", ">>".dark_cyan(), name)?;
            }

            Ok(())
        }
    }

    fn height(&self) -> usize {
        if self.expanded {
            self.list.height() + 1
        } else if self.input.value().is_some() {
            self.input.height() + 1
        } else {
            self.input.height()
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        if self.input.handle_key(key) {
            self.list.list.selected = self.input.value();
            true
        } else if self.expanded {
            self.list.handle_key(key)
        } else {
            false
        }
    }

    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        if self.expanded {
            let w = self.input.cursor_pos(ANSWER_PROMPT.len() as u16).0;
            (w, self.height() as u16)
        } else {
            self.input.cursor_pos(prompt_len)
        }
    }
}

impl Expand {
    fn render_choice<W: std::io::Write>(
        &self,
        item: &ExpandItem,
        max_width: usize,
        w: &mut W,
    ) -> crossterm::Result<()> {
        let hovered = self.selected.map(|c| c == item.key).unwrap_or(false);

        if hovered {
            queue!(w, SetForegroundColor(Color::DarkCyan))?;
        }

        write!(w, "  {}) ", item.key)?;
        item.name.as_str().render(max_width - 5, w)?;

        if hovered {
            queue!(w, ResetColor)?;
        }

        Ok(())
    }
}

thread_local! {
    static HELP_CHOICE: ExpandItem = ExpandItem {
        key: 'h',
        name: "Help, list all options".into(),
    };
}

impl widgets::List for Expand {
    fn render_item<W: std::io::Write>(
        &mut self,
        index: usize,
        _: bool,
        max_width: usize,
        w: &mut W,
    ) -> crossterm::Result<()> {
        if index == self.choices.len() {
            return HELP_CHOICE.with(|h| self.render_choice(h, max_width, w));
        }

        match &self.choices[index] {
            Choice::Choice(item) => self.render_choice(item, max_width, w),
            Choice::Separator(s) => {
                queue!(w, SetForegroundColor(Color::DarkGrey))?;
                w.write_all(b"   ")?;
                super::get_sep_str(s).render(max_width - 3, w)?;
                queue!(w, ResetColor)
            }
        }
    }

    fn is_selectable(&self, _: usize) -> bool {
        true
    }

    fn len(&self) -> usize {
        self.choices.len() + 1
    }
}

impl Expand {
    pub fn ask<W: std::io::Write>(self, opts: Options, w: &mut W) -> error::Result<Answer> {
        let hint: String = {
            let mut s = String::with_capacity(3 + self.choices.len());
            s.push('(');
            s.extend(
                self.choices
                    .choices
                    .iter()
                    .filter_map(|choice| match choice {
                        Choice::Choice(choice) => Some(choice.key),
                        _ => None,
                    }),
            );
            s += "h)";
            s
        };

        let ans = ui::Input::new(ExpandPrompt {
            input: widgets::CharInput::new(|c| {
                let c = c.to_ascii_lowercase();
                hint[1..(hint.len() - 1)].contains(c).then(|| c)
            }),
            list: widgets::ListPicker::new(self),
            opts,
            hint: &hint,
            expanded: false,
        })
        .run(w)?;

        writeln!(w, "{}", ans.name.as_str().dark_cyan())?;

        Ok(Answer::ExpandItem(ans))
    }
}

impl super::Question {
    pub fn expand(
        name: String,
        message: String,
        choices: Vec<Choice<ExpandItem>>,
        default: Option<char>,
    ) -> Self {
        Self::new(
            name,
            message,
            super::QuestionKind::Expand(Expand {
                choices: super::ChoiceList {
                    choices,
                    default: 0,
                    should_loop: true,
                    // FIXME: this should be something sensible. page size is currently not used so
                    // its fine for now
                    page_size: 0,
                },
                selected: None,
                default,
            }),
        )
    }
}

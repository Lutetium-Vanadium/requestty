mod checkbox;
mod choice;
mod confirm;
mod editor;
mod expand;
mod input;
mod list;
mod number;
#[macro_use]
mod options;
mod password;
mod rawlist;

use crate::{error, Answer};
pub use choice::Choice;
use choice::{get_sep_str, ChoiceList};
// TODO: make this private
pub use options::Options;

use std::io::prelude::*;

pub struct Question<'m, 'w> {
    kind: QuestionKind,
    opts: Options<'m, 'w>,
}

impl<'m, 'w> Question<'m, 'w> {
    fn new(opts: Options<'m, 'w>, kind: QuestionKind) -> Self {
        Self { opts, kind }
    }
}

enum QuestionKind {
    Input(input::Input),
    Int(number::Int),
    Float(number::Float),
    Confirm(confirm::Confirm),
    List(list::List),
    Rawlist(rawlist::Rawlist),
    Expand(expand::Expand),
    Checkbox(checkbox::Checkbox),
    Password(password::Password),
    Editor(editor::Editor),
}

impl Question<'_, '_> {
    pub fn ask<W: Write>(self, w: &mut W) -> error::Result<Option<Answer>> {
        if !self.opts.when.get() {
            return Ok(None);
        }

        let mut name = self.opts.name;
        let message = self
            .opts
            .message
            .map(options::Getter::get)
            .unwrap_or_else(|| {
                name.push(':');
                name
            });

        let res = match self.kind {
            QuestionKind::Input(i) => i.ask(message, w)?,
            QuestionKind::Int(i) => i.ask(message, w)?,
            QuestionKind::Float(f) => f.ask(message, w)?,
            QuestionKind::Confirm(c) => c.ask(message, w)?,
            QuestionKind::List(l) => l.ask(message, w)?,
            QuestionKind::Rawlist(r) => r.ask(message, w)?,
            QuestionKind::Expand(e) => e.ask(message, w)?,
            QuestionKind::Checkbox(c) => c.ask(message, w)?,
            QuestionKind::Password(p) => p.ask(message, w)?,
            QuestionKind::Editor(e) => e.ask(message, w)?,
        };

        Ok(Some(res))
    }
}

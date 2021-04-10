mod checkbox;
mod choice_list;
mod confirm;
mod editor;
mod expand;
mod input;
mod list;
mod number;
mod password;
mod raw_list;

use crate::{error, Answer};
pub use choice_list::Choice;
use choice_list::{get_sep_str, ChoiceList};

use std::io::prelude::*;

pub struct Options {
    // FIXME: reference instead?
    name: String,
    // FIXME: reference instead? Dynamic messages?
    message: String,
    // FIXME: Wrong type
    when: bool,
}

pub struct Question {
    kind: QuestionKind,
    opts: Options,
}

impl Question {
    fn new(name: String, message: String, kind: QuestionKind) -> Self {
        Self {
            opts: Options {
                name,
                message,
                when: true,
            },
            kind,
        }
    }
}

enum QuestionKind {
    Input(input::Input),
    Int(number::Int),
    Float(number::Float),
    Confirm(confirm::Confirm),
    List(list::List),
    Rawlist(raw_list::Rawlist),
    Expand(expand::Expand),
    Checkbox(checkbox::Checkbox),
    Password(password::Password),
    Editor(editor::Editor),
}

impl Question {
    pub fn ask<W: Write>(self, w: &mut W) -> error::Result<Answer> {
        match self.kind {
            QuestionKind::Input(i) => i.ask(self.opts, w),
            QuestionKind::Int(i) => i.ask(self.opts, w),
            QuestionKind::Float(f) => f.ask(self.opts, w),
            QuestionKind::Confirm(c) => c.ask(self.opts, w),
            QuestionKind::List(l) => l.ask(self.opts, w),
            QuestionKind::Rawlist(r) => r.ask(self.opts, w),
            QuestionKind::Expand(e) => e.ask(self.opts, w),
            QuestionKind::Checkbox(c) => c.ask(self.opts, w),
            QuestionKind::Password(p) => p.ask(self.opts, w),
            QuestionKind::Editor(e) => e.ask(self.opts, w),
        }
    }
}

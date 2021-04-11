// FIXME: remove this
#![allow(dead_code)]

mod answer;
mod error;
pub mod question;

pub use answer::{Answer, ExpandItem, ListItem};
pub use question::{Choice, Choice::Separator, Question};

pub struct Inquisition<'m, 'w> {
    questions: Vec<Question<'m, 'w>>,
}

impl<'m, 'w> Inquisition<'m, 'w> {
    pub fn new(questions: Vec<Question<'m, 'w>>) -> Self {
        Inquisition { questions }
    }

    pub fn add_question(&mut self, question: Question<'m, 'w>) {
        self.questions.push(question)
    }

    pub fn prompt(self) -> PromptModule<'m, 'w> {
        PromptModule {
            answers: Vec::with_capacity(self.questions.len()),
            questions: self.questions,
        }
    }
}

// TODO: ask questions
pub struct PromptModule<'m, 'w> {
    questions: Vec<Question<'m, 'w>>,
    answers: Vec<Answer>,
}

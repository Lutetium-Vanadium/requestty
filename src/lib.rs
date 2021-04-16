// FIXME: remove this
#![allow(dead_code)]

mod answer;
mod error;
pub mod question;

pub use answer::{Answer, ExpandItem, ListItem};
pub use question::{Choice, Choice::Separator, Question};

pub struct Inquisition<'m, 'w, 'f, 'v, 't> {
    questions: Vec<Question<'m, 'w, 'f, 'v, 't>>,
}

impl<'m, 'w, 'f, 'v, 't> Inquisition<'m, 'w, 'f, 'v, 't> {
    pub fn new(questions: Vec<Question<'m, 'w, 'f, 'v, 't>>) -> Self {
        Inquisition { questions }
    }

    pub fn add_question(&mut self, question: Question<'m, 'w, 'f, 'v, 't>) {
        self.questions.push(question)
    }

    pub fn prompt(self) -> PromptModule<'m, 'w, 'f, 'v, 't> {
        PromptModule {
            answers: Vec::with_capacity(self.questions.len()),
            questions: self.questions,
        }
    }
}

// TODO: ask questions
pub struct PromptModule<'m, 'w, 'f, 'v, 't> {
    questions: Vec<Question<'m, 'w, 'f, 'v, 't>>,
    answers: Vec<Answer>,
}

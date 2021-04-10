// FIXME: remove this
#![allow(dead_code)]

mod answer;
mod error;
pub mod question;

pub use answer::{Answer, ExpandItem, ListItem};
pub use question::{Choice, Choice::Separator, Question};

pub struct Inquisition {
    questions: Vec<Question>,
}

impl Inquisition {
    pub fn new(questions: Vec<Question>) -> Self {
        Inquisition { questions }
    }

    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question)
    }

    pub fn prompt(self) -> PromptModule {
        PromptModule {
            answers: Vec::with_capacity(self.questions.len()),
            questions: self.questions,
        }
    }
}

// TODO: ask questions
pub struct PromptModule {
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

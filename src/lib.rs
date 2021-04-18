use std::{borrow::Borrow, hash::Hash};

use fxhash::FxHashMap as HashMap;

mod answer;
mod error;
mod question;

pub use answer::{Answer, ExpandItem, ListItem};
pub use question::{Choice::Choice, Choice::Separator, Question};

#[derive(Debug, Default)]
pub struct Answers {
    answers: HashMap<String, Answer>,
}

impl Answers {
    fn insert(&mut self, name: String, answer: Answer) {
        self.answers.insert(name, answer);
    }

    fn reserve(&mut self, capacity: usize) {
        self.answers.reserve(capacity - self.answers.len())
    }

    pub fn contains<Q: ?Sized>(&self, question: &Q) -> bool
    where
        String: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.answers.contains_key(question)
    }
}

pub struct PromptModule<'m, 'w, 'f, 'v, 't> {
    questions: Vec<Question<'m, 'w, 'f, 'v, 't>>,
    answers: Answers,
}

impl<'m, 'w, 'f, 'v, 't> PromptModule<'m, 'w, 'f, 'v, 't> {
    pub fn new(questions: Vec<Question<'m, 'w, 'f, 'v, 't>>) -> Self {
        Self {
            answers: Answers::default(),
            questions,
        }
    }

    pub fn with_answers(mut self, answers: Answers) -> Self {
        self.answers = answers;
        self
    }

    pub fn prompt_all(self) -> error::Result<Answers> {
        let PromptModule {
            questions,
            mut answers,
        } = self;

        answers.reserve(questions.len());

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        for question in questions {
            if let Some((name, answer)) = question.ask(&answers, &mut stdout)? {
                answers.insert(name, answer);
            }
        }

        Ok(answers)
    }
}

pub fn prompt(questions: Vec<Question>) -> error::Result<Answers> {
    PromptModule::new(questions).prompt_all()
}

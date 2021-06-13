mod answer;
pub mod question;

use ui::{backend, error, events};

pub use answer::{Answer, Answers, ExpandItem, ListItem};
pub use macros::questions;
pub use question::{
    Choice::Choice, Choice::DefaultSeparator, Choice::Separator, Question,
};
pub use ui::error::{ErrorKind, Result};

#[macro_export]
macro_rules! prompt_module {
    ($($tt:tt)*) => {
        $crate::PromptModule::new($crate::questions! [ $($tt)* ])
    };
}

pub mod plugin {
    pub use crate::{question::Plugin, Answer, Answers};
    pub use ui::{self, backend::Backend, events::Events};
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromptModule<Q> {
    questions: Q,
    answers: Answers,
}

impl<'a, Q> PromptModule<Q>
where
    Q: Iterator<Item = Question<'a>>,
{
    pub fn new<I>(questions: I) -> Self
    where
        I: IntoIterator<IntoIter = Q, Item = Question<'a>>,
    {
        Self {
            answers: Answers::default(),
            questions: questions.into_iter(),
        }
    }

    pub fn with_answers(mut self, answers: Answers) -> Self {
        self.answers = answers;
        self
    }

    fn prompt_impl<B: backend::Backend>(
        &mut self,
        stdout: &mut B,
        events: &mut events::Events,
    ) -> error::Result<Option<&mut Answer>> {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) =
                question.ask(&self.answers, stdout, events)?
            {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    pub fn prompt(&mut self) -> error::Result<Option<&mut Answer>> {
        if atty::isnt(atty::Stream::Stdout) {
            return Err(error::ErrorKind::NotATty);
        }
        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;

        self.prompt_impl(&mut stdout, &mut events::Events::new())
    }

    pub fn prompt_all(mut self) -> error::Result<Answers> {
        self.answers.reserve(self.questions.size_hint().0);

        if atty::isnt(atty::Stream::Stdout) {
            return Err(error::ErrorKind::NotATty);
        }
        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;

        let mut events = events::Events::new();

        while self.prompt_impl(&mut stdout, &mut events)?.is_some() {}

        Ok(self.answers)
    }

    pub fn into_answers(self) -> Answers {
        self.answers
    }
}

pub fn prompt<'a, Q>(questions: Q) -> error::Result<Answers>
where
    Q: IntoIterator<Item = Question<'a>>,
{
    PromptModule::new(questions).prompt_all()
}

pub fn prompt_one<'a, I: Into<Question<'a>>>(question: I) -> error::Result<Answer> {
    let ans = prompt(std::iter::once(question.into()))?;
    Ok(ans.into_iter().next().unwrap().1)
}

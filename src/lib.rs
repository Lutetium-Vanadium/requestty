mod answer;
pub mod question;

use ui::{backend, events};

pub use answer::{Answer, Answers, ExpandItem, ListItem};
pub use macros::questions;
pub use question::{Choice::Choice, Choice::DefaultSeparator, Choice::Separator, Question};
pub use ui::error::{ErrorKind, Result};

#[macro_export]
macro_rules! prompt_module {
    ($($tt:tt)*) => {
        $crate::PromptModule::new($crate::questions! [ $($tt)* ])
    };
}

pub mod plugin {
    pub use crate::{question::Plugin, Answer, Answers};
    pub use ui::{self, backend::Backend, events::KeyEvent};
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

    pub fn prompt_with<B, E>(
        &mut self,
        backend: &mut B,
        events: &mut E,
    ) -> Result<Option<&mut Answer>>
    where
        B: backend::Backend,
        E: Iterator<Item = Result<events::KeyEvent>>,
    {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) = question.ask(&self.answers, backend, events)? {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    pub fn prompt(&mut self) -> Result<Option<&mut Answer>> {
        if atty::isnt(atty::Stream::Stdout) {
            return Err(ErrorKind::NotATty);
        }
        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;

        self.prompt_with(&mut stdout, &mut events::Events::new())
    }

    pub fn prompt_all_with<B, E>(mut self, backend: &mut B, events: &mut E) -> Result<Answers>
    where
        B: backend::Backend,
        E: Iterator<Item = Result<events::KeyEvent>>,
    {
        self.answers.reserve(self.questions.size_hint().0);

        while self.prompt_with(backend, events)?.is_some() {}

        Ok(self.answers)
    }

    pub fn prompt_all(self) -> Result<Answers> {
        if atty::isnt(atty::Stream::Stdout) {
            return Err(ErrorKind::NotATty);
        }

        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;
        let mut events = events::Events::new();

        self.prompt_all_with(&mut stdout, &mut events)
    }

    pub fn into_answers(self) -> Answers {
        self.answers
    }
}

pub fn prompt<'a, Q>(questions: Q) -> Result<Answers>
where
    Q: IntoIterator<Item = Question<'a>>,
{
    PromptModule::new(questions).prompt_all()
}

pub fn prompt_one<'a, I: Into<Question<'a>>>(question: I) -> Result<Answer> {
    let ans = prompt(std::iter::once(question.into()))?;
    Ok(ans.into_iter().next().unwrap().1)
}

pub fn prompt_with<'a, Q, B, E>(questions: Q, backend: &mut B, events: &mut E) -> Result<Answers>
where
    Q: IntoIterator<Item = Question<'a>>,
    B: backend::Backend,
    E: Iterator<Item = Result<events::KeyEvent>>,
{
    PromptModule::new(questions).prompt_all_with(backend, events)
}

pub fn prompt_one_with<'a, Q, B, E>(question: Q, backend: &mut B, events: &mut E) -> Result<Answer>
where
    Q: Into<Question<'a>>,
    B: backend::Backend,
    E: Iterator<Item = Result<events::KeyEvent>>,
{
    let ans = prompt_with(std::iter::once(question.into()), backend, events)?;
    Ok(ans.into_iter().next().unwrap().1)
}

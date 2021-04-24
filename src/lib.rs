mod answer;
mod error;
mod question;

use crossterm::tty::IsTty;

pub use answer::{Answer, Answers, ExpandItem, ListItem};
pub use question::{Choice::Choice, Choice::Separator, Plugin, Question};

pub mod plugin {
    pub use crate::Plugin;
    pub use ui;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromptModule<Q> {
    questions: Q,
    answers: Answers,
}

impl<'m, 'w, 'f, 'v, 't, Q> PromptModule<Q>
where
    Q: Iterator<Item = Question<'m, 'w, 'f, 'v, 't>>,
{
    pub fn new<I>(questions: I) -> Self
    where
        I: IntoIterator<IntoIter = Q>,
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

    fn prompt_impl<W: std::io::Write>(
        &mut self,
        stdout: &mut W,
    ) -> error::Result<Option<&mut Answer>> {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) = question.ask(&self.answers, stdout)? {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    pub fn prompt(&mut self) -> error::Result<Option<&mut Answer>> {
        let stdout = std::io::stdout();
        if !stdout.is_tty() {
            return Err(error::InquirerError::NotATty);
        }
        let mut stdout = stdout.lock();
        self.prompt_impl(&mut stdout)
    }

    pub fn prompt_all(mut self) -> error::Result<Answers> {
        self.answers.reserve(self.questions.size_hint().0);

        let stdout = std::io::stdout();
        if !stdout.is_tty() {
            return Err(error::InquirerError::NotATty);
        }
        let mut stdout = stdout.lock();

        while self.prompt_impl(&mut stdout)?.is_some() {}

        Ok(self.answers)
    }

    cfg_async! {
    async fn prompt_impl_async<W: std::io::Write>(
        &mut self,
        stdout: &mut W,
    ) -> error::Result<Option<&mut Answer>> {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) = question.ask_async(&self.answers, stdout).await? {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    pub async fn prompt_async(&mut self) -> error::Result<Option<&mut Answer>> {
        let stdout = std::io::stdout();
        if !stdout.is_tty() {
            return Err(error::InquirerError::NotATty);
        }
        let mut stdout = stdout.lock();
        self.prompt_impl_async(&mut stdout).await
    }

    pub async fn prompt_all_async(mut self) -> error::Result<Answers> {
        self.answers.reserve(self.questions.size_hint().0);

        let stdout = std::io::stdout();
        if !stdout.is_tty() {
            return Err(error::InquirerError::NotATty);
        }
        let mut stdout = stdout.lock();

        while self.prompt_impl_async(&mut stdout).await?.is_some() {}

        Ok(self.answers)
    }
    }

    pub fn into_answers(self) -> Answers {
        self.answers
    }
}

pub fn prompt<'m, 'w, 'f, 'v, 't, Q>(questions: Q) -> error::Result<Answers>
where
    Q: IntoIterator<Item = Question<'m, 'w, 'f, 'v, 't>>,
{
    PromptModule::new(questions).prompt_all()
}

/// Sets the exit handler to call when `CTRL+C` or EOF is received
///
/// By default, it exits the program, however it can be overridden to not exit. If it doesn't exit,
/// [`Input::run`] will return an `Err`
pub fn set_exit_handler(handler: fn()) {
    ui::set_exit_handler(handler);
}

#[doc(hidden)]
#[macro_export]
macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "async_tokio", feature = "async_std", feature = "async_smol"))]
            $item
        )*
    };
}

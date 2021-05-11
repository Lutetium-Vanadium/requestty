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
    pub use ui::{backend::Backend, events::Events};
    crate::cfg_async! {
    pub use ui::events::AsyncEvents;
    }
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

    cfg_async! {
    async fn prompt_impl_async<B: backend::Backend>(
        &mut self,
        stdout: &mut B,
        events: &mut events::AsyncEvents,
    ) -> error::Result<Option<&mut Answer>> {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) = question.ask_async(&self.answers, stdout, events).await? {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    pub async fn prompt_async(&mut self) -> error::Result<Option<&mut Answer>> {
        if atty::isnt(atty::Stream::Stdout) {
            return Err(error::ErrorKind::NotATty);
        }
        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;

        self.prompt_impl_async(&mut stdout, &mut events::AsyncEvents::new().await?).await
    }

    pub async fn prompt_all_async(mut self) -> error::Result<Answers> {
        self.answers.reserve(self.questions.size_hint().0);

        if atty::isnt(atty::Stream::Stdout) {
            return Err(error::ErrorKind::NotATty);
        }
        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;

        let mut events = events::AsyncEvents::new().await?;

        while self.prompt_impl_async(&mut stdout, &mut events).await?.is_some() {}

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

cfg_async! {
pub async fn prompt_async<'m, 'w, 'f, 'v, 't, Q>(questions: Q) -> error::Result<Answers>
where
    Q: IntoIterator<Item = Question<'m, 'w, 'f, 'v, 't>>,
{
    PromptModule::new(questions).prompt_all_async().await
}
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
            #[cfg(any(feature = "tokio", feature = "async-std", feature = "smol"))]
            $item
        )*
    };
}

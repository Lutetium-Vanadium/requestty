use ui::{
    backend::{get_backend, Backend},
    events::{get_events, EventIterator},
};

use crate::{Answer, Answers, Question};

/// A collection of questions and answers for previously answered questions.
///
/// Unlike [`prompt`], this allows you to control how many questions you want to ask, and ask with
/// previous answers as well.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptModule<Q> {
    questions: Q,
    answers: Answers,
}

impl<'a, Q> PromptModule<Q>
where
    Q: Iterator<Item = Question<'a>>,
{
    /// Creates a new `PromptModule` with the given questions
    pub fn new<I>(questions: I) -> Self
    where
        I: IntoIterator<IntoIter = Q, Item = Question<'a>>,
    {
        Self {
            answers: Answers::default(),
            questions: questions.into_iter(),
        }
    }

    /// Creates a `PromptModule` with the given questions and answers
    pub fn with_answers(mut self, answers: Answers) -> Self {
        self.answers = answers;
        self
    }

    /// Prompt a single question with the default [`Backend`] and [`EventIterator`].
    ///
    /// This may or may not actually prompt the question based on what `when` and `ask_if_answered`
    /// is set to for that particular question.
    pub fn prompt(&mut self) -> crate::Result<Option<&mut Answer>> {
        let stdout = std::io::stdout();
        let mut stdout = get_backend(stdout.lock())?;

        self.prompt_with(&mut stdout, &mut get_events())
    }

    /// Prompt a single question with the given [`Backend`] and [`EventIterator`].
    ///
    /// This may or may not actually prompt the question based on what `when` and `ask_if_answered`
    /// is set to for that particular question.
    pub fn prompt_with<B, E>(
        &mut self,
        backend: &mut B,
        events: &mut E,
    ) -> crate::Result<Option<&mut Answer>>
    where
        B: Backend,
        E: EventIterator,
    {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) = question.ask(&self.answers, backend, events)? {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    /// Prompt all remaining questions with the default [`Backend`] and [`EventIterator`].
    ///
    /// It consumes `self` and returns the answers to all the questions asked.
    pub fn prompt_all(self) -> crate::Result<Answers> {
        let stdout = std::io::stdout();
        let mut stdout = get_backend(stdout.lock())?;
        let mut events = get_events();

        self.prompt_all_with(&mut stdout, &mut events)
    }

    /// Prompt all remaining questions with the given [`Backend`] and [`EventIterator`].
    ///
    /// It consumes `self` and returns the answers to all the questions asked.
    pub fn prompt_all_with<B, E>(
        mut self,
        backend: &mut B,
        events: &mut E,
    ) -> crate::Result<Answers>
    where
        B: Backend,
        E: EventIterator,
    {
        self.answers.reserve(self.questions.size_hint().0);

        while self.prompt_with(backend, events)?.is_some() {}

        Ok(self.answers)
    }

    /// Consumes `self` returning the answers to the previously asked questions.
    pub fn into_answers(self) -> Answers {
        self.answers
    }
}

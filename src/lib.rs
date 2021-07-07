mod answer;
pub mod question;

use ui::{
    backend,
    events::{self, EventIterator},
};

pub use answer::{Answer, Answers, ExpandItem, ListItem};
pub use question::{Choice::Choice, Choice::DefaultSeparator, Choice::Separator, Question};
pub use ui::{ErrorKind, Result};

/// A macro to easily write an iterator of [`Question`]s.
///
/// # Usage
///
/// You can specify questions similar to a `vec![]` of struct instantiations. Each field corresponds
/// to calling the builder method of the same name for the respective question kind.
/// ```
/// # let some_variable = "message";
/// # let when = true;
/// # fn get_default() -> bool { true }
/// use discourse::questions;
///
/// let questions = questions![
///     MultiSelect {
///         // Each field takes a value, which can result in anything that implements `Into`
///         // for the required type
///         name: "name",
///         // The value can be any expression, for example a local variable.
///         message: some_variable,
///         // If the field name and the variable name are the same, this shorthand can be
///         // used.
///         when,
///         // While most values are generic expressions, if a array literal is passed to
///         // choices, some special syntax applies. Also, unlike other fields, 'choices'
///         // will call `choices_with_default` for `MultiSelect` questions only.
///         choices: [
///             // By default array entries are taken as `Choice(_)`s.
///             "Choice 1",
///             // For `MultiSelect` if the word 'default' follows the initial expression, a
///             // default can be set for that choice.
///             "Choice 2" default get_default(),
///             // The word 'separator' or 'sep' can be used to create separators. If no
///             // expression is given along with 'separator', it is taken as a
///             // `DefaultSeparator`.
///             separator,
///             // Otherwise if there is an expression, it is taken as a `Separator(_)`,
///             sep "Separator text!",
///         ],
///     },
/// ];
/// ```
///
/// # Inline
///
/// By default, the questions are stored in a [`Vec`]. However, if you wish to store the questions
/// on the stack, prefix the questions with `inline`:
/// ```
/// use discourse::questions;
///
/// let questions = questions! [ inline
///     Input {
///         name: "input"
///     },
/// ];
/// ```
///
/// Note that inlining only works for rust version 1.51 onwards. Pre 1.51, a [`Vec`] is still used.
///
/// See also [`prompt_module`].
pub use macros::questions;

/// A macro to easily get a [`PromptModule`].
///
/// # Usage
///
/// You can specify questions similar to a `vec![]` of struct instantiations. Each field corresponds
/// to calling the builder method of the same name for the respective question kind.
/// ```
/// # let some_variable = "message";
/// # let when = true;
/// # fn get_default() -> bool { true }
/// use discourse::prompt_module;
///
/// let prompt_module = prompt_module![
///     MultiSelect {
///         // Each field takes a value, which can result in anything that implements `Into`
///         // for the required type
///         name: "name",
///         // The value can be any expression, for example a local variable.
///         message: some_variable,
///         // If the field name and the variable name are the same, this shorthand can be
///         // used.
///         when,
///         // While most values are generic expressions, if a array literal is passed to
///         // choices, some special syntax applies. Also, unlike other fields, 'choices'
///         // will call `choices_with_default` for `MultiSelect` questions only.
///         choices: [
///             // By default array entries are taken as `Choice(_)`s.
///             "Choice 1",
///             // For `MultiSelect` if the word 'default' follows the initial expression, a
///             // default can be set for that choice.
///             "Choice 2" default get_default(),
///             // The word 'separator' or 'sep' can be used to create separators. If no
///             // expression is given along with 'separator', it is taken as a
///             // `DefaultSeparator`.
///             separator,
///             // Otherwise if there is an expression, it is taken as a `Separator(_)`,
///             sep "Separator text!",
///         ],
///     },
/// ];
/// ```
///
/// # Inline
///
/// By default, the questions are stored in a [`Vec`]. However, if you wish to store the questions
/// on the stack, prefix the questions with `inline`:
/// ```
/// use discourse::prompt_module;
///
/// let prompt_module = prompt_module![ inline
///     Input {
///         name: "input"
///     },
/// ];
/// ```
///
/// Note that inlining only works for rust version 1.51 onwards. Pre 1.51, a [`Vec`] is still used.
///
/// See also [`questions`].
#[macro_export]
macro_rules! prompt_module {
    ($($tt:tt)*) => {
        $crate::PromptModule::new($crate::questions! [ $($tt)* ])
    };
}

pub mod plugin {
    pub use crate::{question::Plugin, Answer, Answers};
    pub use ui::{self, backend::Backend, events::EventIterator};
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
        E: EventIterator,
    {
        while let Some(question) = self.questions.next() {
            if let Some((name, answer)) = question.ask(&self.answers, backend, events)? {
                return Ok(Some(self.answers.insert(name, answer)));
            }
        }

        Ok(None)
    }

    pub fn prompt(&mut self) -> Result<Option<&mut Answer>> {
        let stdout = std::io::stdout();
        let mut stdout = backend::get_backend(stdout.lock())?;

        self.prompt_with(&mut stdout, &mut events::Events::new())
    }

    pub fn prompt_all_with<B, E>(mut self, backend: &mut B, events: &mut E) -> Result<Answers>
    where
        B: backend::Backend,
        E: EventIterator,
    {
        self.answers.reserve(self.questions.size_hint().0);

        while self.prompt_with(backend, events)?.is_some() {}

        Ok(self.answers)
    }

    pub fn prompt_all(self) -> Result<Answers> {
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
    E: EventIterator,
{
    PromptModule::new(questions).prompt_all_with(backend, events)
}

pub fn prompt_one_with<'a, Q, B, E>(question: Q, backend: &mut B, events: &mut E) -> Result<Answer>
where
    Q: Into<Question<'a>>,
    B: backend::Backend,
    E: EventIterator,
{
    let ans = prompt_with(std::iter::once(question.into()), backend, events)?;
    Ok(ans.into_iter().next().unwrap().1)
}

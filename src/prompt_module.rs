use ui::{backend::Backend, events::EventIterator};

use crate::{Answer, Answers, Question};

/// A collection of questions and answers for previously answered questions.
///
/// Unlike [`prompt`], this allows you to control how many questions you want to ask, and ask with
/// previous answers as well.
///
/// [`prompt`]: crate::prompt()
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
    /// returns for that particular question.
    #[cfg(any(feature = "crossterm", feature = "termion"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
    pub fn prompt(&mut self) -> crate::Result<Option<&mut Answer>> {
        let stdout = std::io::stdout();
        let mut stdout = ui::backend::get_backend(stdout.lock());

        self.prompt_with(&mut stdout, &mut ui::events::get_events())
    }

    /// Prompt a single question with the given [`Backend`] and [`EventIterator`].
    ///
    /// This may or may not actually prompt the question based on what `when` and `ask_if_answered`
    /// returns for that particular question.
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
    #[cfg(any(feature = "crossterm", feature = "termion"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
    pub fn prompt_all(self) -> crate::Result<Answers> {
        let stdout = std::io::stdout();
        let mut stdout = ui::backend::get_backend(stdout.lock());
        let mut events = ui::events::get_events();

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

/// A macro to easily write a [`PromptModule`].
///
/// # Usage
///
/// You can specify questions similar to a `vec![]` of struct instantiations. Each field corresponds
/// to calling the builder method of the same name for the respective question kind.
/// ```
/// # let some_variable = "message";
/// # let when = true;
/// # fn get_default() -> bool { true }
/// use requestty::prompt_module;
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
///             // expression is given along with 'separator', it is taken as a `DefaultSeparator`.
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
/// use requestty::prompt_module;
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
///
/// [`questions`]: crate::questions
#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[macro_export]
macro_rules! prompt_module {
    ($($tt:tt)*) => {
        $crate::PromptModule::new($crate::questions! [ $($tt)* ])
    };
}

use ui::{backend::Backend, events::EventIterator};

use super::{Options, Question, QuestionKind};
use crate::{Answer, Answers};

/// Prompts are a way to write custom [`Question`]s.
///
/// The prompt is given a `message`, the previous [`Answers`] and a [`Backend`] and
/// [`EventIterator`]. Using these, it is responsible for doing everything from rendering to user
/// interaction. While no particular look is enforced, it is recommended to keep a similar look to
/// the rest of the in-built questions.
///
/// You can use the `requestty-ui` crate to build the prompts. You can see the implementations of
/// the in-built questions for examples on how to use it.
///
/// See also [`Question::prompt`]
pub trait Prompt: std::fmt::Debug {
    /// Prompt the user with the given message, [`Answers`], [`Backend`] and [`EventIterator`]
    fn ask(
        self,
        message: String,
        answers: &Answers,
        backend: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer>;
}

/// The same trait as `Prompt`, except it take `&mut self` instead of `self`.
///
/// This is required since traits with functions that take `self` are not object safe, and so
/// implementors of the trait would have to use &mut self even though it will only be called once.
///
/// Now instead of QuestionKind::Cusrom having a `dyn Prompt`, it has a `dyn CustomPromptInteral`, which
/// is an `Option<T: Prompt>`.
pub(super) trait CustomPromptInteral: std::fmt::Debug {
    fn ask(
        &mut self,
        message: String,
        answers: &Answers,
        backend: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer>;
}

impl<T: Prompt> CustomPromptInteral for Option<T> {
    fn ask(
        &mut self,
        message: String,
        answers: &Answers,
        backend: &mut dyn Backend,
        events: &mut dyn EventIterator,
    ) -> ui::Result<Answer> {
        self.take()
            .expect("Prompt::ask called twice")
            .ask(message, answers, backend, events)
    }
}

/// The builder for custom questions.
///
/// See [`Prompt`] for more information on writing custom prompts.
///
/// # Examples
///
/// ```
/// use requestty::{prompt, Question};
///
/// #[derive(Debug)]
/// struct MyPrompt { /* ... */ }
///
/// # impl MyPrompt {
/// #     fn new() -> MyPrompt {
/// #         MyPrompt {}
/// #     }
/// # }
///
/// impl prompt::Prompt for MyPrompt {
///     fn ask(
///         self,
///         message: String,
///         answers: &prompt::Answers,
///         backend: &mut dyn prompt::Backend,
///         events: &mut dyn prompt::EventIterator,
///     ) -> requestty::Result<prompt::Answer> {
///         // ...
/// #         todo!()
///     }
/// }
///
/// let prompt = Question::custom("my-prompt", MyPrompt::new())
///     .message("Hello from MyPrompt!")
///     .build();
/// ```
#[derive(Debug)]
pub struct CustomPromptBuilder<'a> {
    opts: Options<'a>,
    prompt: Box<dyn CustomPromptInteral + 'a>,
}

impl<'a> CustomPromptBuilder<'a> {
    pub(super) fn new(name: String, prompt: Box<dyn CustomPromptInteral + 'a>) -> Self {
        Self {
            opts: Options::new(name),
            prompt,
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::{prompt, Question};
    ///
    /// #[derive(Debug)]
    /// struct MyPrompt { /* ... */ }
    ///
    /// # impl MyPrompt {
    /// #     fn new() -> MyPrompt {
    /// #         MyPrompt {}
    /// #     }
    /// # }
    ///
    /// impl prompt::Prompt for MyPrompt {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &prompt::Answers,
    ///         backend: &mut dyn prompt::Backend,
    ///         events: &mut dyn prompt::EventIterator,
    ///     ) -> requestty::Result<prompt::Answer> {
    ///         // ...
    /// #         todo!()
    ///     }
    /// }
    ///
    /// let prompt = Question::custom("my-prompt", MyPrompt::new())
    ///     .message("Hello from MyPrompt!")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{prompt, Question, Answers};
    ///
    /// #[derive(Debug)]
    /// struct MyPrompt { /* ... */ }
    ///
    /// # impl MyPrompt {
    /// #     fn new() -> MyPrompt {
    /// #         MyPrompt {}
    /// #     }
    /// # }
    ///
    /// impl prompt::Prompt for MyPrompt {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &prompt::Answers,
    ///         backend: &mut dyn prompt::Backend,
    ///         events: &mut dyn prompt::EventIterator,
    ///     ) -> requestty::Result<prompt::Answer> {
    ///         // ...
    /// #         todo!()
    ///     }
    /// }
    ///
    /// let prompt = Question::custom("my-prompt", MyPrompt::new())
    ///     .when(|previous_answers: &Answers| match previous_answers.get("use-custom-prompt") {
    ///         Some(ans) => !ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use requestty::{prompt, Question};
    ///
    /// #[derive(Debug)]
    /// struct MyPrompt { /* ... */ }
    ///
    /// # impl MyPrompt {
    /// #     fn new() -> MyPrompt {
    /// #         MyPrompt {}
    /// #     }
    /// # }
    ///
    /// impl prompt::Prompt for MyPrompt {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &prompt::Answers,
    ///         backend: &mut dyn prompt::Backend,
    ///         events: &mut dyn prompt::EventIterator,
    ///     ) -> requestty::Result<prompt::Answer> {
    ///         // ...
    /// #         todo!()
    ///     }
    /// }
    ///
    /// let prompt = Question::custom("my-prompt", MyPrompt::new())
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Consumes the builder returning a [`Question`]
    pub fn build(self) -> Question<'a> {
        Question::new(self.opts, QuestionKind::Custom(self.prompt))
    }
}

impl<'a> From<CustomPromptBuilder<'a>> for Question<'a> {
    /// Consumes the builder returning a [`Question`]
    fn from(builder: CustomPromptBuilder<'a>) -> Self {
        builder.build()
    }
}

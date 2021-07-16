use ui::backend::Backend;

use super::Input;
use crate::question::{Completions, Options};

/// The builder for an [`input`] prompt.
///
/// <img
///   src="https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/input.gif"
///   style="max-height: 11rem"
/// />
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use requestty::Question;
///
/// let input = Question::input("name")
///     .message("What is your name?")
///     .default("John Doe")
///     .transform(|name, previous_answers, backend| {
///         write!(backend, "Hello, {}!", name)
///     })
///     .build();
/// ```
///
/// [`input`]: crate::question::Question::input
#[derive(Debug)]
pub struct InputBuilder<'a> {
    opts: Options<'a>,
    input: Input<'a>,
}

impl<'a> InputBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        InputBuilder {
            opts: Options::new(name),
            input: Default::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .message("What is your name?")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let input = Question::input("name")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("anonymous") {
    ///         Some(ans) => !ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let input = Question::input("name")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Set a default value for the input
    ///
    /// If set and the user presses `Enter` without typing any text, the `default` is taken as the
    /// answer.
    ///
    /// If `default` is used, validation is skipped, but `filter` is still called.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .default("John Doe")
    ///     .build();
    /// ```
    pub fn default<I: Into<String>>(mut self, default: I) -> Self {
        self.input.default = Some(default.into());
        self
    }

    crate::impl_auto_complete_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, question::completions};
    ///
    /// let input = Question::input("name")
    ///     .auto_complete(|name, previous_answers| {
    ///         completions![name, "John Doe".to_owned()]
    ///     })
    ///     .build();
    /// ```
    ///
    /// For a better example on `auto_complete`, see [`examples/file.rs`]
    ///
    /// [`examples/file.rs`]: https://github.com/Lutetium-Vanadium/requestty/blob/master/examples/file.rs
    String; input
    }

    /// The maximum height that can be taken by the [`auto_complete`] selection list
    ///
    /// If the total height exceeds the page size, the list will be scrollable.
    ///
    /// The `page_size` must be a minimum of 5. If `page_size` is not set, it will default to 15. It
    /// will only be used if [`auto_complete`] is set, and returns more than 1 completions.
    ///
    /// [`auto_complete`]: InputBuilder::auto_complete
    ///
    /// # Panics
    ///
    /// It will panic if the `page_size` is less than 5.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.input.page_size = page_size;
        self
    }

    /// Whether to wrap around when user gets to the last element.
    ///
    /// If `should_loop` is not set, it will default to `true`. It will only be used if
    /// [`auto_complete`] is set, and returns more than 1 completions.
    ///
    /// [`auto_complete`]: InputBuilder::auto_complete
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.input.should_loop = should_loop;
        self
    }

    crate::impl_filter_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .filter(|name, previous_answers| name + "!")
    ///     .build();
    /// ```
    String; input
    }

    crate::impl_validate_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .validate(|name, previous_answers| if name.split_whitespace().count() >= 2 {
    ///         Ok(())
    ///     } else {
    ///         Err("Please enter your first and last name".to_owned())
    ///     })
    ///     .build();
    /// ```
    str; input
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let input = Question::input("name")
    ///     .transform(|name, previous_answers, backend| {
    ///         write!(backend, "Hello, {}!", name)
    ///     })
    ///     .build();
    /// ```
    str; input
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> crate::question::Question<'a> {
        crate::question::Question::new(self.opts, crate::question::QuestionKind::Input(self.input))
    }
}

impl<'a> From<InputBuilder<'a>> for crate::question::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: InputBuilder<'a>) -> Self {
        builder.build()
    }
}

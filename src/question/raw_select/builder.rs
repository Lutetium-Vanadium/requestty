use ui::{backend::Backend, widgets::Text};

use super::RawSelect;
use crate::{
    question::{Choice, Options},
    ListItem,
};

/// The builder for a [`raw_select`] prompt.
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use requestty::{Question, DefaultSeparator};
///
/// let raw_select = Question::raw_select("theme")
///     .message("What do you want to do?")
///     .choices(vec![
///         "Order a pizza".into(),
///         "Make a reservation".into(),
///         DefaultSeparator,
///         "Ask for opening hours".into(),
///         "Contact support".into(),
///         "Talk to the receptionist".into(),
///     ])
///     .build();
/// ```
///
/// [`raw_select`]: crate::question::Question::raw_select
#[derive(Debug)]
pub struct RawSelectBuilder<'a> {
    opts: Options<'a>,
    raw_select: RawSelect<'a>,
    choice_count: usize,
}

impl<'a> RawSelectBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        RawSelectBuilder {
            opts: Options::new(name),
            raw_select: Default::default(),
            // It is one indexed for the user
            choice_count: 1,
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .message("What do you want to do?")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("use-default-theme") {
    ///         Some(ans) => ans.as_bool().unwrap(),
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
    /// let raw_select = Question::raw_select("theme")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Set a default index for the select
    ///
    /// The given index will be hovered in the beginning.
    ///
    /// If `default` is unspecified, the first [`Choice`] will be hovered.
    ///
    /// # Panics
    ///
    /// If the default given is not a [`Choice`], it will cause a panic on [`build`]
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`build`]: Self::build
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, DefaultSeparator};
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .choices(vec![
    ///         "Order a pizza".into(),
    ///         "Make a reservation".into(),
    ///         DefaultSeparator,
    ///         "Ask for opening hours".into(),
    ///         "Contact support".into(),
    ///         "Talk to the receptionist".into(),
    ///     ])
    ///     .default(1)
    ///     .build();
    /// ```
    pub fn default(mut self, default: usize) -> Self {
        self.raw_select.choices.set_default(default);
        self
    }

    /// The maximum height that can be taken by the list
    ///
    /// If the total height exceeds the page size, the list will be scrollable.
    ///
    /// The `page_size` must be a minimum of 5. If `page_size` is not set, it will default to 15.
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
    /// let raw_select = Question::raw_select("theme")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.raw_select.choices.set_page_size(page_size);
        self
    }

    /// Whether to wrap around when user gets to the last element.
    ///
    /// This only applies when the list is scrollable, i.e. page size > total height.
    ///
    /// If `should_loop` is not set, it will default to `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.raw_select.choices.set_should_loop(should_loop);
        self
    }

    /// Inserts a [`Choice`].
    ///
    /// See [`raw_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice::Choice
    /// [`raw_select`]: crate::question::Question::raw_select
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .choice("Order a Pizza")
    ///     .build();
    /// ```
    pub fn choice<I: Into<String>>(mut self, choice: I) -> Self {
        self.raw_select.choices.choices.push(Choice::Choice((
            self.choice_count,
            Text::new(choice.into()),
        )));
        self.choice_count += 1;
        self
    }

    /// Inserts a [`Separator`] with the given text
    ///
    /// See [`raw_select`] for more information.
    ///
    /// [`Separator`]: crate::question::Choice::Separator
    /// [`raw_select`]: crate::question::Question::raw_select
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .separator("-- custom separator text --")
    ///     .build();
    /// ```
    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.raw_select
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self
    }

    /// Inserts a [`DefaultSeparator`]
    ///
    /// See [`raw_select`] for more information.
    ///
    /// [`DefaultSeparator`]: crate::question::Choice::DefaultSeparator
    /// [`raw_select`]: crate::question::Question::raw_select
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .default_separator()
    ///     .build();
    /// ```
    pub fn default_separator(mut self) -> Self {
        self.raw_select
            .choices
            .choices
            .push(Choice::DefaultSeparator);
        self
    }

    /// Extends the given iterator of [`Choice`]s
    ///
    /// See [`raw_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`raw_select`]: crate::question::Question::raw_select
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, DefaultSeparator};
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .choices(vec![
    ///         "Order a pizza".into(),
    ///         "Make a reservation".into(),
    ///         DefaultSeparator,
    ///         "Ask for opening hours".into(),
    ///         "Contact support".into(),
    ///         "Talk to the receptionist".into(),
    ///     ])
    ///     .build();
    /// ```
    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<String>>,
        I: IntoIterator<Item = T>,
    {
        let choice_count = &mut self.choice_count;
        self.raw_select
            .choices
            .choices
            .extend(choices.into_iter().map(|choice| {
                choice.into().map(|c| {
                    let choice = (*choice_count, Text::new(c));
                    *choice_count += 1;
                    choice
                })
            }));
        self
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .transform(|choice, previous_answers, backend| {
    ///         write!(backend, "({}) {}", choice.index, choice.name)
    ///     })
    ///     .build();
    /// ```
    ListItem; raw_select
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> crate::question::Question<'a> {
        crate::question::Question::new(
            self.opts,
            crate::question::QuestionKind::RawSelect(self.raw_select),
        )
    }
}

impl<'a> From<RawSelectBuilder<'a>> for crate::question::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: RawSelectBuilder<'a>) -> Self {
        builder.build()
    }
}

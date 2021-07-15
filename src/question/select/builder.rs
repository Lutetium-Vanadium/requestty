use ui::{backend::Backend, widgets::Text};

use super::Select;
use crate::{
    question::{Choice, Options},
    ListItem,
};

/// The builder for a [`select`] prompt.
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use discourse::{Question, DefaultSeparator};
///
/// let select = Question::select("theme")
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
/// [`select`]: crate::question::Question::select
#[derive(Debug)]
pub struct SelectBuilder<'a> {
    opts: Options<'a>,
    select: Select<'a>,
}

impl<'a> SelectBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        SelectBuilder {
            opts: Options::new(name),
            select: Default::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let select = Question::select("theme")
    ///     .message("What do you want to do?")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use discourse::{Question, Answers};
    ///
    /// let select = Question::select("theme")
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
    /// use discourse::{Question, Answers};
    ///
    /// let select = Question::select("theme")
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
    /// use discourse::{Question, DefaultSeparator};
    ///
    /// let select = Question::select("theme")
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
        self.select.choices.set_default(default);
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
    /// use discourse::Question;
    ///
    /// let select = Question::select("theme")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.select.choices.set_page_size(page_size);
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
    /// use discourse::Question;
    ///
    /// let select = Question::select("theme")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.select.choices.set_should_loop(should_loop);
        self
    }

    /// Inserts a [`Choice`].
    ///
    /// See [`select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice::Choice
    /// [`select`]: crate::question::Question::select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let select = Question::select("theme")
    ///     .choice("Order a Pizza")
    ///     .build();
    /// ```
    pub fn choice<I: Into<String>>(mut self, choice: I) -> Self {
        self.select
            .choices
            .choices
            .push(Choice::Choice(Text::new(choice.into())));
        self
    }

    /// Inserts a [`Separator`] with the given text
    ///
    /// See [`select`] for more information.
    ///
    /// [`Separator`]: crate::question::Choice::Separator
    /// [`select`]: crate::question::Question::select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let select = Question::select("theme")
    ///     .separator("-- custom separator text --")
    ///     .build();
    /// ```
    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.select
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self
    }

    /// Inserts a [`DefaultSeparator`]
    ///
    /// See [`select`] for more information.
    ///
    /// [`DefaultSeparator`]: crate::question::Choice::DefaultSeparator
    /// [`select`]: crate::question::Question::select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let select = Question::select("theme")
    ///     .default_separator()
    ///     .build();
    /// ```
    pub fn default_separator(mut self) -> Self {
        self.select.choices.choices.push(Choice::DefaultSeparator);
        self
    }

    /// Extends the given iterator of [`Choice`]s
    ///
    /// See [`select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`select`]: crate::question::Question::select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::{Question, DefaultSeparator};
    ///
    /// let select = Question::select("theme")
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
        self.select.choices.choices.extend(
            choices
                .into_iter()
                .map(|choice| choice.into().map(Text::new)),
        );
        self
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let raw_select = Question::raw_select("theme")
    ///     .transform(|choice, previous_answers, backend| {
    ///         write!(backend, "({}) {}", choice.index, choice.name)
    ///     })
    ///     .build();
    /// ```
    ListItem; select
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> crate::question::Question<'a> {
        if let Some(default) = self.select.choices.default() {
            if self.select.choices[default].is_separator() {
                panic!("Invalid default '{}' is not a `Choice`", default);
            }
        }

        crate::question::Question::new(
            self.opts,
            crate::question::QuestionKind::Select(self.select),
        )
    }
}

impl<'a> From<SelectBuilder<'a>> for crate::question::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: SelectBuilder<'a>) -> Self {
        builder.build()
    }
}
use ui::{backend::Backend, widgets::Text};

use super::MultiSelect;
use crate::{
    question::{Choice, Options},
    ListItem,
};

/// The builder for a [`multi_select`] prompt.
///
/// Unlike the other list based prompts, this has a per choice boolean default.
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use discourse::{Question, DefaultSeparator};
///
/// let multi_select = Question::multi_select("cheese")
///     .message("What cheese do you want?")
///     .choice_with_default("Mozzarella", true)
///     .choices(vec![
///         "Cheddar",
///         "Parmesan",
///     ])
///     .build();
/// ```
///
/// [`multi_select`]: crate::question::Question::multi_select
#[derive(Debug)]
pub struct MultiSelectBuilder<'a> {
    opts: Options<'a>,
    multi_select: MultiSelect<'a>,
}

impl<'a> MultiSelectBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        MultiSelectBuilder {
            opts: Options::new(name),
            multi_select: Default::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .message("What cheese do you want?")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use discourse::{Answers, Question};
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("vegan") {
    ///         Some(ans) => ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use discourse::{Answers, Question};
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
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
    /// let multi_select = Question::multi_select("cheese")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.multi_select.choices.set_page_size(page_size);
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
    /// let multi_select = Question::multi_select("cheese")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.multi_select.choices.set_should_loop(should_loop);
        self
    }

    /// Inserts a [`Choice`] with its default checked state as `false`.
    ///
    /// If you want to set the default checked state, use [`choice_with_default`].
    ///
    /// See [`multi_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice::Choice
    /// [`choice_with_default`]: Self::choice_with_default
    /// [`multi_select`]: crate::question::Question::multi_select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .choice("Cheddar")
    ///     .build();
    /// ```
    pub fn choice<I: Into<String>>(self, choice: I) -> Self {
        self.choice_with_default(choice.into(), false)
    }

    /// Inserts a [`Choice`] with a given default checked state.
    ///
    /// See [`multi_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice::Choice
    /// [`multi_select`]: crate::question::Question::multi_select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .choice_with_default("Mozzarella", true)
    ///     .build();
    /// ```
    pub fn choice_with_default<I: Into<String>>(mut self, choice: I, default: bool) -> Self {
        self.multi_select
            .choices
            .choices
            .push(Choice::Choice(Text::new(choice.into())));
        self.multi_select.selected.push(default);
        self
    }

    /// Inserts a [`Separator`] with the given text
    ///
    /// See [`multi_select`] for more information.
    ///
    /// [`Separator`]: crate::question::Choice::Separator
    /// [`multi_select`]: crate::question::Question::multi_select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .separator("-- custom separator text --")
    ///     .build();
    /// ```
    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.multi_select
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self.multi_select.selected.push(false);
        self
    }

    /// Inserts a [`DefaultSeparator`]
    ///
    /// See [`multi_select`] for more information.
    ///
    /// [`DefaultSeparator`]: crate::question::Choice::DefaultSeparator
    /// [`multi_select`]: crate::question::Question::multi_select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .default_separator()
    ///     .build();
    /// ```
    pub fn default_separator(mut self) -> Self {
        self.multi_select
            .choices
            .choices
            .push(Choice::DefaultSeparator);
        self.multi_select.selected.push(false);
        self
    }

    /// Extends the given iterator of [`Choice`]s
    ///
    /// Every [`Choice::Choice`] within will have a default checked value of `false`. If you want to
    /// set the default checked value, use [`choices_with_default`].
    ///
    /// See [`multi_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`choices_with_default`]: Self::choices_with_default
    /// [`multi_select`]: crate::question::Question::multi_select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .choices(vec![
    ///         "Mozzarella",
    ///         "Cheddar",
    ///         "Parmesan",
    ///     ])
    ///     .build();
    /// ```
    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<String>>,
        I: IntoIterator<Item = T>,
    {
        self.multi_select
            .choices
            .choices
            .extend(choices.into_iter().map(|c| c.into().map(Text::new)));
        self.multi_select
            .selected
            .resize(self.multi_select.choices.len(), false);
        self
    }

    /// Extends the given iterator of [`Choice`]s with the given default checked value.
    ///
    /// See [`multi_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`multi_select`]: crate::question::Question::multi_select
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .choices_with_default(vec![
    ///         ("Mozzarella", true),
    ///         ("Cheddar", false),
    ///         ("Parmesan", false),
    ///     ])
    ///     .build();
    /// ```
    pub fn choices_with_default<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<(String, bool)>>,
        I: IntoIterator<Item = T>,
    {
        let iter = choices.into_iter();
        self.multi_select
            .selected
            .reserve(iter.size_hint().0.saturating_add(1));
        self.multi_select
            .choices
            .choices
            .reserve(iter.size_hint().0.saturating_add(1));

        for choice in iter {
            match choice.into() {
                Choice::Choice((choice, selected)) => {
                    self.multi_select
                        .choices
                        .choices
                        .push(Choice::Choice(Text::new(choice)));
                    self.multi_select.selected.push(selected);
                }
                Choice::Separator(s) => {
                    self.multi_select.choices.choices.push(Choice::Separator(s));
                    self.multi_select.selected.push(false);
                }
                Choice::DefaultSeparator => {
                    self.multi_select
                        .choices
                        .choices
                        .push(Choice::DefaultSeparator);
                    self.multi_select.selected.push(false);
                }
            }
        }
        self
    }

    crate::impl_filter_builder! {
    /// NOTE: The boolean [`Vec`] contains a boolean value for each index even if it is a separator.
    /// However it is guaranteed that all the separator indices will be false.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("evil-cheese")
    ///     .filter(|mut cheeses, previous_answers| {
    ///         cheeses.iter_mut().for_each(|checked| *checked = !*checked);
    ///         cheeses
    ///     })
    ///     .build();
    /// ```
    Vec<bool>; multi_select
    }
    crate::impl_validate_builder! {
    /// NOTE: The boolean [`slice`] contains a boolean value for each index even if it is a
    /// separator. However it is guaranteed that all the separator indices will be false.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .validate(|cheeses, previous_answers| {
    ///         if cheeses.iter().filter(|&&a| a).count() < 1 {
    ///             Err("You must choose at least one cheese.".into())
    ///         } else {
    ///             Ok(())
    ///         }
    ///     })
    ///     .build();
    /// ```
    [bool]; multi_select
    }

    crate::impl_transform_builder! {
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let multi_select = Question::multi_select("cheese")
    ///     .transform(|cheeses, previous_answers, backend| {
    ///         for cheese in cheeses {
    ///             write!(backend, "({}) {}, ", cheese.index, cheese.name)?;
    ///         }
    ///         Ok(())
    ///     })
    ///     .build();
    /// ```
    [ListItem]; multi_select
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> crate::question::Question<'a> {
        crate::question::Question::new(
            self.opts,
            crate::question::QuestionKind::MultiSelect(self.multi_select),
        )
    }
}

impl<'a> From<MultiSelectBuilder<'a>> for crate::question::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: MultiSelectBuilder<'a>) -> Self {
        builder.build()
    }
}

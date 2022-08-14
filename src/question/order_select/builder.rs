use ui::backend::Backend;
use ui::widgets::Text;

use crate::{
    question::{Choice, Options},
    ListItem,
};

use super::OrderSelect;

/// Prompt that allows the user to organize a list of options.
///
/// The choices are represented with the [`Choice`] enum. [`Choice::Choice`] can be multi-line.
/// but [`Choice::Separator`]s will be ignored.
///
/// // TODO : add gif
/// <img
///   src="https://raw.githubusercontent.com/lutetium-vanadium/requestty/master/assets/multi-select.gif"
///   style="max-height: 20rem"
/// />
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// let order_select = Question::order_select("tasks")
///     .message("Please organize the tasks")
///     .choices(vec![
///         "Task 1",
///         "Task 2",
///         "Task 3",
///     ])
///     .build();
/// ```
///
///
/// [`order_select`]: crate::question::Question::order_select
#[derive(Debug)]
pub struct OrderSelectBuilder<'a> {
    opts: Options<'a>,
    order_select: OrderSelect<'a>,
}

impl<'a> OrderSelectBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        Self {
            opts: Options::new(name),
            order_select: Default::default(),
        }
    }

    crate::impl_options_builder! {
        message
        /// # Examples
        ///
        /// ```
        /// use requestty::Question;
        ///
        /// let order_select = Question::order_select("cheese")
        ///     .message("Organize the cheeses")
        ///     ...
        ///     .build();
        /// ```

        when
        /// # Examples
        ///
        /// ```
        /// use requestty::{Answers, Question};
        ///
        /// let order_select = Question::order_select("cheese")
        ///     ...
        ///     .when(|previous_answers: &Answers| match previous_answers.get("vegan") {
        ///         Some(ans) => ans.as_bool().unwrap(),
        ///         None => true,
        ///     })
        ///     ...
        ///     .build();
        /// ```

        ask_if_answered
        /// # Examples
        ///
        /// ```
        /// use requestty::{Answers, Question};
        ///
        /// let order_select = Question::order_select("cheese")
        ///     ...
        ///     .ask_if_answered(true)
        ///     ...
        ///     .build();
        /// ```

        on_esc
        /// # Examples
        ///
        /// ```
        /// use requestty::{Answers, Question, OnEsc};
        ///
        /// let order_select = Question::order_select("cheese")
        ///     ...
        ///     .on_esc(OnEsc::Terminate)
        ///     ...
        ///     .build();
        /// ```
    }

    crate::impl_filter_builder! {
        /// NOTE: The boolean [`Vec`] contains a boolean value for each index even if it is a separator.
        /// However it is guaranteed that all the separator indices will be false.
        ///
        /// # Examples
        ///
        /// ```
        /// use requestty::Question;
        ///
        /// let order_select = Question::order_select("evil-cheese")
        ///     ...
        ///     .filter(|mut cheeses, previous_answers| {
        ///         cheeses.iter_mut().for_each(|checked| *checked = !*checked);
        ///         cheeses
        ///     })
        ///     ...
        ///     .build();
        /// ```
        Vec<usize>; order_select
    }

    crate::impl_validate_builder! {
        /// NOTE: The boolean [`slice`] contains a boolean value for each index even if it is a
        /// separator. However it is guaranteed that all the separator indices will be false.
        ///
        /// # Examples
        ///
        /// ```
        /// use requestty::Question;
        ///
        /// let order_select = Question::order_select("cheese")
        ///     ...
        ///     .validate(|cheeses, previous_answers| {
        ///         if cheeses.iter().filter(|&&a| a).count() < 1 {
        ///             Err("You must choose at least one cheese.".into())
        ///         } else {
        ///             Ok(())
        ///         }
        ///     })
        ///     ...
        ///     .build();
        /// ```
        [usize]; order_select
    }

    crate::impl_transform_builder! {
        /// # Examples
        ///
        /// ```
        /// use requestty::Question;
        ///
        /// let order_select = Question::order_select("cheese")
        ///     .transform(|cheeses, previous_answers, backend| {
        ///         for cheese in cheeses {
        ///             write!(backend, "({}) {}, ", cheese.index, cheese.text)?;
        ///         }
        ///         Ok(())
        ///     })
        ///     .build();
        /// ```
        [ListItem]; order_select
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
    /// let order_select = Question::order_select("cheese")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.order_select.choices.set_page_size(page_size);
        self
    }

    /// Whether to wrap around when user gets to the last element.
    ///
    /// If `should_loop` is not set, it will default to `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let order_select = Question::order_select("cheese")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.order_select.choices.set_should_loop(should_loop);
        self
    }

    // TODO : add docs
    /// Extends the given iterator of [`Choice`]s
    ///
    /// All separators will be ignored.
    ///
    /// See [`order_select`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`order_select`]: crate::question::Question::order_select
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let order_select = Question::order_select("cheese")
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
        self.order_select.choices.choices.extend(
            choices
                .into_iter()
                .filter_map(|c| {
                    if let Choice::Choice(txt) = c.into() {
                        Some(txt)
                    } else {
                        None
                    }
                })
                .map(|c| Choice::Choice(c).map(Text::new)),
        );
        self
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(mut self) -> crate::question::Question<'a> {
        self.order_select.order = (0..self.order_select.choices.len()).collect();
        self.order_select.max_number_len = (self.order_select.order.len() + 1).to_string().len();

        crate::question::Question::new(
            self.opts,
            crate::question::QuestionKind::OrderSelect(self.order_select),
        )
    }
}

impl<'a> From<OrderSelectBuilder<'a>> for crate::question::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: OrderSelectBuilder<'a>) -> Self {
        builder.build()
    }
}
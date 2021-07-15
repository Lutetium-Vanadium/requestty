use std::collections::HashSet;

use ui::{backend::Backend, widgets::Text};

use super::{Expand, ExpandText};
use crate::{
    question::{Choice, Options},
    ExpandItem,
};

/// The builder for a [`expand`] prompt.
///
/// See the various methods for more details on each available option.
///
/// # Examples
///
/// ```
/// use requestty::Question;
///
/// let expand = Question::expand("overwrite")
///     .message("Conflict on `file.rs`")
///     .choices(vec![
///         ('y', "Overwrite"),
///         ('a', "Overwrite this one and all next"),
///         ('d', "Show diff"),
///     ])
///     .default_separator()
///     .choice('x', "Abort")
///     .build();
/// ```
///
/// [`expand`]: crate::question::Question::expand
#[derive(Debug)]
pub struct ExpandBuilder<'a> {
    opts: Options<'a>,
    expand: Expand<'a>,
    keys: HashSet<char>,
}

impl<'a> ExpandBuilder<'a> {
    pub(crate) fn new(name: String) -> Self {
        ExpandBuilder {
            opts: Options::new(name),
            expand: Default::default(),
            keys: HashSet::default(),
        }
    }

    crate::impl_options_builder! {
    message
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .message("Conflict on `file.rs`")
    ///     .build();
    /// ```

    when
    /// # Examples
    ///
    /// ```
    /// use requestty::{Question, Answers};
    ///
    /// let expand = Question::expand("overwrite")
    ///     .when(|previous_answers: &Answers| match previous_answers.get("ignore-conflicts") {
    ///         Some(ans) => ans.as_bool().unwrap(),
    ///         None => true,
    ///     })
    ///     .build();
    /// ```

    ask_if_answered
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .ask_if_answered(true)
    ///     .build();
    /// ```
    }

    /// Set a default key for the expand
    ///
    /// If no key is entered by the user and they press `Enter`, the default key is used.
    ///
    /// If `default` is unspecified, it defaults to the 'h' key.
    ///
    /// # Panics
    ///
    /// If the default given is not a key, it will cause a panic on [`build`]
    ///
    /// [`build`]: Self::build
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .choice('d', "Show diff")
    ///     .default('d')
    ///     .build();
    /// ```
    pub fn default(mut self, default: char) -> Self {
        self.expand.default = default;
        self
    }

    /// The maximum height that can be taken by the expanded list
    ///
    /// If the total height exceeds the page size, the list will be scrollable.
    ///
    /// The `page_size` must be a minimum of 5. If `page_size` is not set, it will default to 15. It
    /// will only be used if the user expands the prompt.
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
    /// let expand = Question::expand("overwrite")
    ///     .page_size(10)
    ///     .build();
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        assert!(page_size >= 5, "page size can be a minimum of 5");

        self.expand.choices.set_page_size(page_size);
        self
    }

    /// Whether to wrap around when user gets to the last element.
    ///
    /// This only applies when the list is scrollable, i.e. page size > total height.
    ///
    /// If `should_loop` is not set, it will default to `true`. It will only be used if the user
    /// expands the prompt.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .should_loop(false)
    ///     .build();
    /// ```
    pub fn should_loop(mut self, should_loop: bool) -> Self {
        self.expand.choices.set_should_loop(should_loop);
        self
    }

    /// Inserts a [`Choice`] with the given key
    ///
    /// See [`expand`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice::Choice
    /// [`expand`]: crate::question::Question::expand
    ///
    /// # Panics
    ///
    /// It will panic if the key is 'h' or a duplicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .choice('x', "Abort")
    ///     .build();
    /// ```
    pub fn choice<I: Into<String>>(mut self, mut key: char, name: I) -> Self {
        key = key.to_ascii_lowercase();

        if key == 'h' {
            panic!("Reserved key 'h'");
        }
        if self.keys.contains(&key) {
            panic!("Duplicate key '{}'", key);
        }

        self.keys.insert(key);

        self.expand.choices.choices.push(Choice::Choice(ExpandText {
            key,
            name: Text::new(name.into()),
        }));

        self
    }

    /// Inserts a [`Separator`] with the given text
    ///
    /// See [`expand`] for more information.
    ///
    /// [`Separator`]: crate::question::Choice::Separator
    /// [`expand`]: crate::question::Question::expand
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .separator("-- custom separator text --")
    ///     .build();
    /// ```
    pub fn separator<I: Into<String>>(mut self, text: I) -> Self {
        self.expand
            .choices
            .choices
            .push(Choice::Separator(text.into()));
        self
    }

    /// Inserts a [`DefaultSeparator`]
    ///
    /// See [`expand`] for more information.
    ///
    /// [`DefaultSeparator`]: crate::question::Choice::DefaultSeparator
    /// [`expand`]: crate::question::Question::expand
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .default_separator()
    ///     .build();
    /// ```
    pub fn default_separator(mut self) -> Self {
        self.expand.choices.choices.push(Choice::DefaultSeparator);
        self
    }

    /// Extends the given iterator of [`Choice`]s
    ///
    /// See [`expand`] for more information.
    ///
    /// [`Choice`]: crate::question::Choice
    /// [`expand`]: crate::question::Question::expand
    ///
    /// # Panics
    ///
    /// It will panic if the key of any choice is 'h' or a duplicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use requestty::Question;
    ///
    /// let expand = Question::expand("overwrite")
    ///     .choices(vec![
    ///         ('y', "Overwrite"),
    ///         ('a', "Overwrite this one and all next"),
    ///         ('d', "Show diff"),
    ///     ])
    ///     .build();
    /// ```
    pub fn choices<I, T>(mut self, choices: I) -> Self
    where
        T: Into<Choice<ExpandItem>>,
        I: IntoIterator<Item = T>,
    {
        let Self {
            ref mut keys,
            ref mut expand,
            ..
        } = self;

        expand.choices.choices.extend(choices.into_iter().map(|c| {
            c.into().map(|ExpandItem { name, mut key }| {
                key = key.to_ascii_lowercase();
                if key == 'h' {
                    panic!("Reserved key 'h'");
                }
                if keys.contains(&key) {
                    panic!("Duplicate key '{}'", key);
                }
                keys.insert(key);

                ExpandText {
                    name: Text::new(name),
                    key,
                }
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
    /// let expand = Question::expand("overwrite")
    ///     .transform(|choice, previous_answers, backend| {
    ///         write!(backend, "({}) {}", choice.key, choice.name)
    ///     })
    ///     .build();
    /// ```
    ExpandItem; expand
    }

    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    pub fn build(self) -> crate::question::Question<'a> {
        if !self.expand.has_valid_default() {
            panic!(
                "Invalid default '{}' does not occur in the given choices",
                self.expand.default
            );
        }

        crate::question::Question::new(
            self.opts,
            crate::question::QuestionKind::Expand(self.expand),
        )
    }
}

impl<'a> From<ExpandBuilder<'a>> for crate::question::Question<'a> {
    /// Consumes the builder returning a [`Question`]
    ///
    /// [`Question`]: crate::question::Question
    fn from(builder: ExpandBuilder<'a>) -> Self {
        builder.build()
    }
}

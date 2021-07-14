//! A module that contains things related to [`Question`]s.

mod choice;
mod confirm;
mod editor;
mod expand;
mod input;
mod multi_select;
mod number;
mod select;
#[macro_use]
mod options;
mod password;
mod plugin;
mod raw_select;

pub use choice::Choice;
pub use confirm::ConfirmBuilder;
pub use editor::EditorBuilder;
pub use expand::ExpandBuilder;
pub use input::InputBuilder;
pub use multi_select::MultiSelectBuilder;
pub use number::{FloatBuilder, IntBuilder};
pub use password::PasswordBuilder;
pub use plugin::{Plugin, PluginBuilder};
pub use raw_select::RawSelectBuilder;
pub use select::SelectBuilder;

use std::fmt;
use ui::{backend::Backend, events::EventIterator};

use crate::{Answer, Answers};
use choice::{get_sep_str, ChoiceList};
use options::Options;
use plugin::PluginInteral;

/// A `Question` that can be asked.
///
/// There are 11 variants.
///
/// - [`input`](Question::input)
/// - [`password`](Question::password)
/// - [`editor`](Question::editor)
/// - [`confirm`](Question::confirm)
/// - [`int`](Question::int)
/// - [`float`](Question::float)
/// - [`expand`](Question::expand)
/// - [`select`](Question::select)
/// - [`raw_select`](Question::raw_select)
/// - [`multi_select`](Question::multi_select)
/// - [`plugin`](Question::plugin)
///
/// Every [`Question`] has 4 common options.
///
/// - `name` (required): This is used as the key in [`Answers`].
///   It is not shown to the user unless `message` is unspecified.
///
/// - `message`: The message to display when the prompt is rendered in the terminal.
///   If it is not given, the `message` defaults to "\<name\>: ". It is recommended to set this as
///   `name` is meant to be a programmatic `id`.
///
/// - `when`: Whether to ask the question or not.
///   This can be used to have context based questions. If it is not given, it defaults to `true`.
///
/// - `ask_if_answered`: Prompt the question even if it is answered.
///   By default if an answer with the given `name` already exists, the question will be skipped.
///   This can be override by setting `ask_if_answered` is set to `true`.
///
/// A `Question` can be asked by creating a [`PromptModule`] or using [`prompt_one`] or
/// [`prompt_one_with`].
///
/// # Examples
///
/// ```
/// use discourse::Question;
///
/// let question = Question::input("name")
///     .message("What is your name?")
///     .default("John Doe")
///     .transform(|name, previous_answers, backend| {
///         write!(backend, "Hello, {}!", name)
///     })
///     .build();
/// ```
///
/// [`PromptModule`]: crate::PromptModule
/// [`prompt_one`]: crate::prompt_one
/// [`prompt_one_with`]: crate::prompt_one_with
#[derive(Debug)]
pub struct Question<'a> {
    kind: QuestionKind<'a>,
    opts: Options<'a>,
}

impl<'a> Question<'a> {
    fn new(opts: Options<'a>, kind: QuestionKind<'a>) -> Self {
        Self { kind, opts }
    }
}

impl Question<'static> {
    /// Prompt that takes user input and returns a [`String`]
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/input.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
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
    /// [`builder`]: InputBuilder
    pub fn input<N: Into<String>>(name: N) -> InputBuilder<'static> {
        InputBuilder::new(name.into())
    }

    /// Prompt that takes user input and hides it.
    ///
    /// How it looks if you set a mask:
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/password-mask.gif)
    ///
    /// How it looks if you do not set a mask:
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/password-hidden.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let password = Question::password("password")
    ///     .message("What is your password?")
    ///     .mask('*')
    ///     .build();
    /// ```
    ///
    /// [`builder`]: PasswordBuilder
    pub fn password<N: Into<String>>(name: N) -> PasswordBuilder<'static> {
        PasswordBuilder::new(name.into())
    }

    /// Prompt that takes launches the users preferred editor on a temporary file
    ///
    /// Once the user exits their editor, the contents of the temporary file are read in as the
    /// result. The editor to use is determined by the `$VISUAL` or `$EDITOR` environment variables.
    /// If neither of those are present, `vim` (for unix) or `notepad` (for windows) is used.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/editor.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let editor = Question::editor("description")
    ///     .message("Please enter a short description about yourself")
    ///     .extension(".md")
    ///     .build();
    /// ```
    ///
    /// [`builder`]: EditorBuilder
    pub fn editor<N: Into<String>>(name: N) -> EditorBuilder<'static> {
        EditorBuilder::new(name.into())
    }

    /// Prompt that returns `true` or `false`.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/confirm.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let confirm = Question::confirm("anonymous")
    ///     .message("Do you want to remain anonymous?")
    ///     .build();
    /// ```
    ///
    /// [`builder`]: ConfirmBuilder
    pub fn confirm<N: Into<String>>(name: N) -> ConfirmBuilder<'static> {
        ConfirmBuilder::new(name.into())
    }

    /// Prompt that takes a [`i64`] as input.
    ///
    /// The number is parsed using [`from_str`].
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/int.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let int = Question::int("age")
    ///     .message("What is your age?")
    ///     .validate(|age, previous_answers| {
    ///         if age > 0 && age < 130 {
    ///             Ok(())
    ///         } else {
    ///             Err(format!("You cannot be {} years old!", age))
    ///         }
    ///     })
    ///     .build();
    /// ```
    ///
    /// [`builder`]: IntBuilder
    /// [`from_str`]: https://doc.rust-lang.org/std/primitive.i64.html#method.from_str
    pub fn int<N: Into<String>>(name: N) -> IntBuilder<'static> {
        IntBuilder::new(name.into())
    }

    /// Prompt that takes a [`f64`] as input.
    ///
    /// The number is parsed using [`from_str`], but cannot be `NaN`.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/float.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
    ///
    /// let float = Question::float("number")
    ///     .message("What is your favourite number?")
    ///     .validate(|num, previous_answers| {
    ///         if num.is_finite() {
    ///             Ok(())
    ///         } else {
    ///             Err("Please enter a finite number".to_owned())
    ///         }
    ///     })
    ///     .build();
    /// ```
    ///
    /// [`builder`]: FloatBuilder
    /// [`from_str`]: https://doc.rust-lang.org/std/primitive.f64.html#method.from_str
    pub fn float<N: Into<String>>(name: N) -> FloatBuilder<'static> {
        FloatBuilder::new(name.into())
    }

    /// Prompt that allows the user to select from a list of options by key
    ///
    /// The keys are ascii case-insensitive characters. The 'h' option is added by the prompt and
    /// shouldn't be defined.
    ///
    /// The choices are represented with the [`Choice`] enum. [`Choice::Choice`] can be multi-line,
    /// but [`Choice::Separator`]s can only be single line.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/expand.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::Question;
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
    /// [`builder`]: ExpandBuilder
    pub fn expand<N: Into<String>>(name: N) -> ExpandBuilder<'static> {
        ExpandBuilder::new(name.into())
    }

    /// Prompt that allows the user to select from a list of options
    ///
    /// The choices are represented with the [`Choice`] enum. [`Choice::Choice`] can be multi-line,
    /// but [`Choice::Separator`]s can only be single line.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/select.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
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
    /// [`builder`]: SelectBuilder
    pub fn select<N: Into<String>>(name: N) -> SelectBuilder<'static> {
        SelectBuilder::new(name.into())
    }

    /// Prompt that allows the user to select from a list of options with indices
    ///
    /// The choices are represented with the [`Choice`] enum. [`Choice::Choice`] can be multi-line,
    /// but [`Choice::Separator`]s can only be single line.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/raw-select.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::{Question, DefaultSeparator};
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
    /// [`builder`]: RawSelectBuilder
    pub fn raw_select<N: Into<String>>(name: N) -> RawSelectBuilder<'static> {
        RawSelectBuilder::new(name.into())
    }

    /// Prompt that allows the user to select multiple items from a list of options
    ///
    /// Unlike the other list based prompts, this has a per choice boolean default.
    ///
    /// The choices are represented with the [`Choice`] enum. [`Choice::Choice`] can be multi-line,
    /// but [`Choice::Separator`]s can only be single line.
    ///
    /// ![](https://raw.githubusercontent.com/lutetium-vanadium/discourse/master/assets/multi-select.gif)
    ///
    /// See the various methods on the [`builder`] for more details on each available option.
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
    /// [`builder`]: MultiSelectBuilder
    pub fn multi_select<N: Into<String>>(name: N) -> MultiSelectBuilder<'static> {
        MultiSelectBuilder::new(name.into())
    }

    /// Create a [`Question`] from a custom prompt.
    ///
    /// See [`Plugin`] for more information on writing custom prompts.
    ///
    /// # Examples
    ///
    /// ```
    /// use discourse::{plugin, Question};
    ///
    /// #[derive(Debug)]
    /// struct MyPlugin { /* ... */ }
    ///
    /// # impl MyPlugin {
    /// #     fn new() -> MyPlugin {
    /// #         MyPlugin {}
    /// #     }
    /// # }
    ///
    /// impl plugin::Plugin for MyPlugin {
    ///     fn ask(
    ///         self,
    ///         message: String,
    ///         answers: &plugin::Answers,
    ///         backend: &mut dyn plugin::Backend,
    ///         events: &mut dyn plugin::EventIterator,
    ///     ) -> discourse::Result<plugin::Answer> {
    /// #       todo!()
    ///         /* ... */
    ///     }
    /// }
    ///
    /// let plugin = Question::plugin("my-plugin", MyPlugin::new())
    ///     .message("Hello from MyPlugin!")
    ///     .build();
    /// ```
    ///
    /// [`builder`]: PluginBuilder
    pub fn plugin<'a, N, P>(name: N, plugin: P) -> PluginBuilder<'a>
    where
        N: Into<String>,
        P: Plugin + 'a,
    {
        PluginBuilder::new(name.into(), Box::new(Some(plugin)))
    }
}

#[derive(Debug)]
enum QuestionKind<'a> {
    Input(input::Input<'a>),
    Int(number::Int<'a>),
    Float(number::Float<'a>),
    Confirm(confirm::Confirm<'a>),
    Select(select::Select<'a>),
    RawSelect(raw_select::RawSelect<'a>),
    Expand(expand::Expand<'a>),
    MultiSelect(multi_select::MultiSelect<'a>),
    Password(password::Password<'a>),
    Editor(editor::Editor<'a>),
    Plugin(Box<dyn PluginInteral + 'a>),
}

impl Question<'_> {
    pub(crate) fn ask<B: Backend, I: EventIterator>(
        mut self,
        answers: &Answers,
        b: &mut B,
        events: &mut I,
    ) -> ui::Result<Option<(String, Answer)>> {
        // Already asked
        if !self.opts.ask_if_answered && answers.contains_key(&self.opts.name) {
            return Ok(None);
        }

        // Shouldn't be asked
        if !self.opts.when.get(answers) {
            return Ok(None);
        }

        let name = self.opts.name;
        let message = self
            .opts
            .message
            .map(|message| message.get(answers))
            .unwrap_or_else(|| name.clone() + ":");

        let res = match self.kind {
            QuestionKind::Input(i) => i.ask(message, answers, b, events)?,
            QuestionKind::Int(i) => i.ask(message, answers, b, events)?,
            QuestionKind::Float(f) => f.ask(message, answers, b, events)?,
            QuestionKind::Confirm(c) => c.ask(message, answers, b, events)?,
            QuestionKind::Select(l) => l.ask(message, answers, b, events)?,
            QuestionKind::RawSelect(r) => r.ask(message, answers, b, events)?,
            QuestionKind::Expand(e) => e.ask(message, answers, b, events)?,
            QuestionKind::MultiSelect(c) => c.ask(message, answers, b, events)?,
            QuestionKind::Password(p) => p.ask(message, answers, b, events)?,
            QuestionKind::Editor(e) => e.ask(message, answers, b, events)?,
            QuestionKind::Plugin(ref mut o) => o.ask(message, answers, b, events)?,
        };

        Ok(Some((name, res)))
    }
}

macro_rules! handler {
    ($name:ident, $fn_trait:ident ( $($type:ty),* ) -> $return:ty) => {
        pub(crate) enum $name<'a, T> {
            Sync(Box<dyn $fn_trait( $($type),* ) -> $return + 'a>),
            None,
        }

        impl<'a, T> $name<'a, T> {
            #[allow(unused)]
            fn take(&mut self) -> Self {
                std::mem::replace(self, Self::None)
            }
        }

        impl<T> Default for $name<'_, T> {
            fn default() -> Self {
                Self::None
            }
        }

        impl<T: fmt::Debug> fmt::Debug for $name<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Sync(_) => f.write_str("Sync(_)"),
                    Self::None => f.write_str("None"),
                }
            }
        }
    };

    // The type signature of the function must only contain &T
    ($name:ident, ?Sized $fn_trait:ident ( $($type:ty),* ) -> $return:ty) => {
        pub(crate) enum $name<'a, T: ?Sized> {
            Sync(Box<dyn $fn_trait( $($type),* ) -> $return + 'a>),
            None,
        }

        impl<'a, T: ?Sized> $name<'a, T> {
            #[allow(unused)]
            fn take(&mut self) -> Self {
                std::mem::replace(self, Self::None)
            }
        }

        impl<T: ?Sized> Default for $name<'_, T> {
            fn default() -> Self {
                Self::None
            }
        }

        impl<T: fmt::Debug + ?Sized> fmt::Debug for $name<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Sync(_) => f.write_str("Sync(_)"),
                    Self::None => f.write_str("None"),
                }
            }
        }
    };
}

/// The type which needs to be returned by the [`auto_complete`] function.
///
/// [`auto_complete`]: InputBuilder::auto_complete
#[cfg(feature = "smallvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec")))]
pub type Completions<T> = smallvec::SmallVec<[T; 1]>;

/// The type which needs to be returned by the [`auto_complete`] function.
///
/// [`auto_complete`]: InputBuilder::auto_complete
#[cfg(not(feature = "smallvec"))]
pub type Completions<T> = Vec<T>;

#[cfg(feature = "smallvec")]
#[cfg_attr(docsrs, doc(cfg(feature = "smallvec")))]
pub use smallvec::smallvec as completions;

#[cfg(not(feature = "smallvec"))]
pub use std::vec as completions;

#[doc(hidden)]
#[macro_export]
macro_rules! __completions_count {
    ($e:expr) => (1);
    ($e:expr, $($rest:expr)+) => (1 + $(+ $crate::question::__completions_count!($rest) )+);
}

handler!(Filter, FnOnce(T, &Answers) -> T);
handler!(AutoComplete, FnMut(T, &Answers) -> Completions<T>);
handler!(Validate, ?Sized FnMut(&T, &Answers) -> Result<(), String>);
handler!(ValidateByVal, FnMut(T, &Answers) -> Result<(), String>);
handler!(Transform, ?Sized FnOnce(&T, &Answers, &mut dyn Backend) -> std::io::Result<()>);
handler!(
    TransformByVal,
    FnOnce(T, &Answers, &mut dyn Backend) -> std::io::Result<()>
);

#[doc(hidden)]
#[macro_export]
macro_rules! impl_filter_builder {
    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        /// Function to change the final submitted value before it is displayed to the user and
        /// added to the [`Answers`].
        ///
        /// It is a [`FnOnce`] that is given the answer and the previous [`Answers`], and should
        /// return the new answer.
        ///
        /// This will be called after the answer has been validated.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$meta])+
        pub fn filter<F>(mut self, filter: F) -> Self
        where
            F: FnOnce($t, &crate::Answers) -> $t + 'a,
        {
            self.$inner.filter = crate::question::Filter::Sync(Box::new(filter));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_auto_complete_builder {
    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        /// Function to suggest completions to the answer when the user presses `Tab`.
        ///
        /// It is a [`FnMut`] that is given the current state of the answer and the previous
        /// [`Answers`], and should return a list of completions.
        ///
        /// There must be at least 1 completion. Returning 0 completions will cause a panic. If
        /// there are no completions to give, you can simply return the state of the answer passed
        /// to you.
        ///
        /// If there is 1 completion, then the state of the answer becomes that completion.
        ///
        /// If there are 2 or more completions, a list of completions is displayed from which the
        /// user can pick one completion.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$meta])+
        pub fn auto_complete<F>(mut self, auto_complete: F) -> Self
        where
            F: FnMut($t, &crate::Answers) -> Completions<$t> + 'a,
        {
            self.$inner.auto_complete =
                crate::question::AutoComplete::Sync(Box::new(auto_complete));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_validate_builder {
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        crate::impl_validate_builder!($(#[$meta])* impl &$t; $inner Validate);
    };

    ($(#[$meta:meta])+ by val $t:ty; $inner:ident) => {
        crate::impl_validate_builder!($(#[$meta])* impl $t; $inner ValidateByVal);
    };

    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ impl $t:ty; $inner:ident $handler:ident) => {
        /// Function to validate the submitted value before it's returned.
        ///
        /// It is a [`FnMut`] that is given the answer and the previous [`Answers`], and should
        /// return `Ok(())` if the given answer is valid. If it is invalid, it should return an
        /// [`Err`] with the error message to display to the user.
        ///
        /// This will be called when the user presses the `Enter` key.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$meta])*
        pub fn validate<F>(mut self, filter: F) -> Self
        where
            F: FnMut($t, &crate::Answers) -> Result<(), String> + 'a,
        {
            self.$inner.validate = crate::question::$handler::Sync(Box::new(filter));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_transform_builder {
    ($(#[$meta:meta])+ $t:ty; $inner:ident) => {
        crate::impl_transform_builder!($(#[$meta])* impl &$t; $inner Transform);
    };

    ($(#[$meta:meta])+ by val $t:ty; $inner:ident) => {
        crate::impl_transform_builder!($(#[$meta])* impl $t; $inner TransformByVal);
    };

    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    ($(#[$meta:meta])+ impl $t:ty; $inner:ident $handler:ident) => {
        /// Change the way the answer looks when displayed to the user.
        ///
        /// It is a [`FnOnce`] that is given the answer, previous [`Answers`] and the [`Backend`] to
        /// display the answer on. After the `transform` is called, a new line is also added.
        ///
        /// It will only be called once the user finishes answering the question.
        ///
        /// [`Answers`]: crate::Answers
        /// [`Backend`]: crate::plugin::Backend
        ///
        ///
        $(#[$meta])*
        pub fn transform<F>(mut self, transform: F) -> Self
        where
            F: FnOnce($t, &crate::Answers, &mut dyn Backend) -> std::io::Result<()> + 'a,
        {
            self.$inner.transform = crate::question::$handler::Sync(Box::new(transform));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! write_final {
    ($transform:expr, $message:expr, $ans:expr, $answers:expr, $backend:expr, $custom:expr) => {{
        ui::widgets::Prompt::write_finished_message(&$message, $backend)?;

        match $transform {
            Transform::Sync(transform) => transform($ans, $answers, $backend)?,
            _ => $custom,
        }

        $backend.write_all(b"\n")?;
        $backend.flush()?;
    }};
}

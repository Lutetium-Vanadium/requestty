use std::fmt;

use ui::OnEsc;

use crate::Answers;

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub(crate) name: String,
    pub(crate) message: Option<Getter<'a, String>>,
    pub(crate) when: Getter<'a, bool>,
    pub(crate) ask_if_answered: bool,
    pub(crate) on_esc: Getter<'a, OnEsc>,
}

impl<'a> Options<'a> {
    pub(crate) fn new(name: String) -> Self {
        Options {
            name,
            message: None,
            when: true.into(),
            ask_if_answered: false,
            on_esc: OnEsc::Ignore.into(),
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_options_builder {
    // NOTE: the 2 extra lines at the end of each doc comment is intentional -- it makes sure that
    // other docs that come from the macro invocation have appropriate spacing
    (message $(#[$message_meta:meta])*
     when $(#[$when_meta:meta])*
     ask_if_answered $(#[$ask_if_answered_meta:meta])*
     $(on_esc $(#[$on_esc_meta:meta])*)?) => {
        /// The message to display when the prompt is rendered in the terminal.
        ///
        /// It can be either a [`String`] or a [`FnOnce`] that returns a [`String`]. If it is a
        /// function, it is passed all the previous [`Answers`], and will be called right before the
        /// question is prompted to the user.
        ///
        /// If it is not given, the `message` defaults to "\<name\>: ".
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$message_meta])*
        pub fn message<M>(mut self, message: M) -> Self
        where
            M: Into<crate::question::options::Getter<'a, String>>,
        {
            self.opts.message = Some(message.into());
            self
        }

        /// Whether to ask the question (`true`) or not (`false`).
        ///
        /// It can be either a [`bool`] or a [`FnOnce`] that returns a [`bool`]. If it is a
        /// function, it is passed all the previous [`Answers`], and will be called right before the
        /// question is prompted to the user.
        ///
        /// If it is not given, it defaults to `true`.
        ///
        /// [`Answers`]: crate::Answers
        ///
        ///
        $(#[$when_meta])*
        pub fn when<W>(mut self, when: W) -> Self
        where
            W: Into<crate::question::options::Getter<'a, bool>>,
        {
            self.opts.when = when.into();
            self
        }

        /// Prompt the question even if it is answered.
        ///
        /// By default if an answer with the given `name` already exists, the question will be
        /// skipped. This can be overridden by setting `ask_if_answered` is set to `true`.
        ///
        /// If this is not given, it defaults to `false`.
        ///
        /// If you need to dynamically decide whether the question should be asked, use [`when`].
        ///
        /// [`Answers`]: crate::Answers
        /// [`when`]: Self::when
        ///
        ///
        $(#[$ask_if_answered_meta])*
        pub fn ask_if_answered(mut self, ask_if_answered: bool) -> Self {
            self.opts.ask_if_answered = ask_if_answered;
            self
        }

        $(
        /// Configure what to do when the user presses the `Esc` key.
        ///
        /// It can be either a [`OnEsc`] or a [`FnOnce`] that returns a [`OnEsc`]. If it is a
        /// function, it is passed all the previous [`Answers`], and will be called right before the
        /// question is prompted to the user.
        ///
        /// If it is not given, it defaults to [`OnEsc::Ignore`].
        ///
        ///
        $(#[$on_esc_meta])*
        pub fn on_esc<T>(mut self, on_esc: T) -> Self
        where
            T: Into<crate::question::options::Getter<'a, ui::OnEsc>>,
        {
            self.opts.on_esc = on_esc.into();
            self
        }
        )?
    };
}

/// Optionally dynamically get a value.
///
/// It can either be a [`FnOnce`] that results in a value, or the value itself.
///
/// This should not need to be constructed manually, as it is used with the [`Into`] trait.
#[allow(missing_docs)]
pub enum Getter<'a, T> {
    Function(Box<dyn FnOnce(&Answers) -> T + 'a>),
    Value(T),
}

impl<T: fmt::Debug> fmt::Debug for Getter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Getter::Function(_) => f.write_str("Function(_)"),
            Getter::Value(v) => write!(f, "Value({:?})", v),
        }
    }
}

impl<T> Getter<'_, T> {
    pub(crate) fn get(self, answers: &Answers) -> T {
        match self {
            Getter::Function(f) => f(answers),
            Getter::Value(v) => v,
        }
    }
}

macro_rules! impl_getter_from_val {
    ($T:ty, $I:ty) => {
        impl_getter_from_val!($T, $I, value => value.into());
    };

    ($T:ty, $I:ty, $value:ident => $body:expr) => {
        impl<'a> From<$I> for Getter<'a, $T> {
            fn from(value: $I) -> Self {
                let $value = value;
                Self::Value($body)
            }
        }
    };
}

impl_getter_from_val!(String, String);
impl_getter_from_val!(String, &String);
impl_getter_from_val!(String, &str);
impl_getter_from_val!(String, &mut str, s => s.to_owned());
impl_getter_from_val!(String, Box<str>);
impl_getter_from_val!(String, char, c => c.to_string());

impl<'a, F> From<F> for Getter<'a, String>
where
    F: FnOnce(&Answers) -> String + 'a,
{
    fn from(f: F) -> Self {
        Getter::Function(Box::new(f))
    }
}

impl_getter_from_val!(bool, bool);
impl<'a, F> From<F> for Getter<'a, bool>
where
    F: FnOnce(&Answers) -> bool + 'a,
{
    fn from(f: F) -> Self {
        Getter::Function(Box::new(f))
    }
}

impl_getter_from_val!(OnEsc, OnEsc);
impl<'a, F> From<F> for Getter<'a, OnEsc>
where
    F: FnOnce(&Answers) -> OnEsc + 'a,
{
    fn from(f: F) -> Self {
        Getter::Function(Box::new(f))
    }
}

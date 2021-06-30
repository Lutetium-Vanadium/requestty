use std::fmt;

use crate::Answers;

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub(crate) name: String,
    pub(crate) message: Option<Getter<'a, String>>,
    pub(crate) when: Getter<'a, bool>,
    pub(crate) ask_if_answered: bool,
}

impl<'a> Options<'a> {
    pub(crate) fn new(name: String) -> Self {
        Options {
            name,
            message: None,
            when: true.into(),
            ask_if_answered: false,
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_options_builder {
    () => {
        pub fn message<M>(mut self, message: M) -> Self
        where
            M: Into<crate::question::options::Getter<'a, String>>,
        {
            self.opts.message = Some(message.into());
            self
        }

        pub fn when<W>(mut self, when: W) -> Self
        where
            W: Into<crate::question::options::Getter<'a, bool>>,
        {
            self.opts.when = when.into();
            self
        }

        pub fn ask_if_answered(mut self, ask_if_answered: bool) -> Self {
            self.opts.ask_if_answered = ask_if_answered;
            self
        }
    };

    ($t:ident; ($self:ident, $opts:ident) => $body:expr) => {
#[rustfmt::skip]
        crate::impl_options_builder!($t<>; ($self, $opts) => $body);
    };
}

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
        impl<'a> From<$I> for Getter<'a, $T> {
            fn from(value: $I) -> Self {
                Self::Value(value.into())
            }
        }
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

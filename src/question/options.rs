use std::fmt;

use crate::Answers;

#[derive(Debug)]
pub(crate) struct Options<'m, 'w> {
    pub(crate) name: String,
    pub(crate) message: Option<Getter<'m, String>>,
    pub(crate) when: Getter<'w, bool>,
    pub(crate) ask_if_answered: bool,
}

impl Options<'static, 'static> {
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
    // Unwieldy macro magic -- matches over lifetimes
    ($ty:ident < $( $lifetime:lifetime ),* >; ($self:ident, $opts:ident) => $body:expr) => {
        impl<'m, 'w, $($lifetime),* > $ty<'m, 'w, $($lifetime),* > {
            pub fn message<'a, M>(self, message: M) -> $ty<'a, 'w, $($lifetime),*>
            where
                M: Into<crate::question::options::Getter<'a, String>>
            {
                let $self = self;
                let $opts = Options {
                    message: Some(message.into()),
                    when: $self.opts.when,
                    name: $self.opts.name,
                    ask_if_answered: $self.opts.ask_if_answered,
                };
                $body
            }

            pub fn when<'a, W>(self, when: W) -> $ty<'m, 'a, $($lifetime),*>
            where
                W: Into<crate::question::options::Getter<'a, bool>>
            {
                let $self = self;
                let $opts = Options {
                    when: when.into(),
                    message: $self.opts.message,
                    name: $self.opts.name,
                    ask_if_answered: $self.opts.ask_if_answered,
                };
                $body
            }

            pub fn ask_if_answered(mut self, ask_if_answered: bool) -> Self {
                self.opts.ask_if_answered = ask_if_answered;
                self
            }
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
        impl From<$I> for Getter<'static, $T> {
            fn from(value: $I) -> Self {
                Self::Value(value.into())
            }
        }
    };
}

impl_getter_from_val!(String, String);
impl_getter_from_val!(String, &String);
impl_getter_from_val!(String, &str);
impl_getter_from_val!(String, &mut str);
impl_getter_from_val!(String, Box<str>);
impl_getter_from_val!(String, char);

impl<'a, F> From<F> for Getter<'a, String>
where
    F: Fn(&Answers) -> String + 'a,
{
    fn from(f: F) -> Self {
        Getter::Function(Box::new(f))
    }
}

impl_getter_from_val!(bool, bool);
impl<'a, F> From<F> for Getter<'a, bool>
where
    F: Fn(&Answers) -> bool + 'a,
{
    fn from(f: F) -> Self {
        Getter::Function(Box::new(f))
    }
}

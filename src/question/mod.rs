mod checkbox;
mod choice;
mod confirm;
mod editor;
mod expand;
mod input;
mod number;
mod select;
#[macro_use]
mod options;
mod password;
mod plugin;
mod raw_select;

pub use checkbox::CheckboxBuilder;
pub use confirm::ConfirmBuilder;
pub use editor::EditorBuilder;
pub use expand::ExpandBuilder;
pub use input::InputBuilder;
pub use number::{FloatBuilder, IntBuilder};
pub use password::PasswordBuilder;
pub use plugin::PluginBuilder;
pub use raw_select::RawSelectBuilder;
pub use select::SelectBuilder;

use crate::{Answer, Answers};
pub use choice::Choice;
use choice::{get_sep_str, ChoiceList};
use options::Options;
pub use plugin::Plugin;
use plugin::PluginInteral;
use ui::{backend::Backend, error, events::KeyEvent};

use std::fmt;

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
    pub fn input<N: Into<String>>(name: N) -> InputBuilder<'static> {
        InputBuilder::new(name.into())
    }

    pub fn int<N: Into<String>>(name: N) -> IntBuilder<'static> {
        IntBuilder::new(name.into())
    }

    pub fn float<N: Into<String>>(name: N) -> FloatBuilder<'static> {
        FloatBuilder::new(name.into())
    }

    pub fn confirm<N: Into<String>>(name: N) -> ConfirmBuilder<'static> {
        ConfirmBuilder::new(name.into())
    }

    pub fn select<N: Into<String>>(name: N) -> SelectBuilder<'static> {
        SelectBuilder::new(name.into())
    }

    pub fn raw_select<N: Into<String>>(name: N) -> RawSelectBuilder<'static> {
        RawSelectBuilder::new(name.into())
    }

    pub fn expand<N: Into<String>>(name: N) -> ExpandBuilder<'static> {
        ExpandBuilder::new(name.into())
    }

    pub fn checkbox<N: Into<String>>(name: N) -> CheckboxBuilder<'static> {
        CheckboxBuilder::new(name.into())
    }

    pub fn password<N: Into<String>>(name: N) -> PasswordBuilder<'static> {
        PasswordBuilder::new(name.into())
    }

    pub fn editor<N: Into<String>>(name: N) -> EditorBuilder<'static> {
        EditorBuilder::new(name.into())
    }

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
    Checkbox(checkbox::Checkbox<'a>),
    Password(password::Password<'a>),
    Editor(editor::Editor<'a>),
    Plugin(Box<dyn PluginInteral + 'a>),
}

impl Question<'_> {
    pub(crate) fn ask<B: Backend, I: Iterator<Item = error::Result<KeyEvent>>>(
        mut self,
        answers: &Answers,
        b: &mut B,
        events: &mut I,
    ) -> error::Result<Option<(String, Answer)>> {
        if (!self.opts.ask_if_answered && answers.contains_key(&self.opts.name))
            || !self.opts.when.get(answers)
        {
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
            QuestionKind::Checkbox(c) => c.ask(message, answers, b, events)?,
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

#[cfg(feature = "smallvec")]
pub type Completions<T> = smallvec::SmallVec<[T; 1]>;
#[cfg(not(feature = "smallvec"))]
pub type Completions<T> = Vec<T>;

handler!(Filter, FnOnce(T, &Answers) -> T);
handler!(AutoComplete, FnMut(T, &Answers) -> Completions<T>);
handler!(Validate, ?Sized FnMut(&T, &Answers) -> Result<(), String>);
handler!(ValidateByVal, FnMut(T, &Answers) -> Result<(), String>);
handler!(Transform, ?Sized FnOnce(&T, &Answers, &mut dyn Backend) -> error::Result<()>);
handler!(
    TransformByVal,
    FnOnce(T, &Answers, &mut dyn Backend) -> error::Result<()>
);

#[doc(hidden)]
#[macro_export]
macro_rules! impl_filter_builder {
    ($t:ty; $inner:ident) => {
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
    ($t:ty; $inner:ident) => {
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
    ($t:ty; $inner:ident) => {
        pub fn validate<F>(mut self, filter: F) -> Self
        where
            F: FnMut(&$t, &crate::Answers) -> Result<(), String> + 'a,
        {
            self.$inner.validate = crate::question::Validate::Sync(Box::new(filter));
            self
        }
    };

    (by val $t:ty; $inner:ident) => {
        pub fn validate<F>(mut self, filter: F) -> Self
        where
            F: FnMut($t, &crate::Answers) -> Result<(), String> + 'a,
        {
            self.$inner.validate = crate::question::ValidateByVal::Sync(Box::new(filter));
            self
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_transform_builder {
    ($t:ty; $inner:ident) => {
        pub fn transform<F>(mut self, transform: F) -> Self
        where
            F: FnOnce(&$t, &crate::Answers, &mut dyn Backend) -> ui::error::Result<()> + 'a,
        {
            self.$inner.transform = crate::question::Transform::Sync(Box::new(transform));
            self
        }
    };

    (by val $t:ty; $inner:ident) => {
        pub fn transform<F>(mut self, transform: F) -> Self
        where
            F: FnOnce($t, &crate::Answers, &mut dyn Backend) -> ui::error::Result<()> + 'a,
        {
            self.$inner.transform = crate::question::TransformByVal::Sync(Box::new(transform));
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

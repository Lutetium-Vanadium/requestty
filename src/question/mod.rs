mod checkbox;
mod choice;
mod confirm;
mod editor;
mod expand;
mod input;
mod list;
mod number;
#[macro_use]
mod options;
mod password;
mod plugin;
mod rawlist;

use crate::{Answer, Answers};
pub use choice::Choice;
use choice::{get_sep_str, ChoiceList};
use options::Options;
pub use plugin::Plugin;
use ui::{backend::Backend, error};

use std::{fmt, future::Future, pin::Pin};

#[derive(Debug)]
pub struct Question<'m, 'w, 'f, 'v, 't> {
    kind: QuestionKind<'f, 'v, 't>,
    opts: Options<'m, 'w>,
}

impl<'m, 'w, 'f, 'v, 't> Question<'m, 'w, 'f, 'v, 't> {
    pub(crate) fn new(
        opts: Options<'m, 'w>,
        kind: QuestionKind<'f, 'v, 't>,
    ) -> Self {
        Self { opts, kind }
    }
}

#[derive(Debug)]
pub(crate) enum QuestionKind<'f, 'v, 't> {
    Input(input::Input<'f, 'v, 't>),
    Int(number::Int<'f, 'v, 't>),
    Float(number::Float<'f, 'v, 't>),
    Confirm(confirm::Confirm<'t>),
    List(list::List<'t>),
    Rawlist(rawlist::Rawlist<'t>),
    Expand(expand::Expand<'t>),
    Checkbox(checkbox::Checkbox<'f, 'v, 't>),
    Password(password::Password<'f, 'v, 't>),
    Editor(editor::Editor<'f, 'v, 't>),
    // random lifetime so that it doesn't have to be static
    Plugin(Box<dyn Plugin + 'f>),
}

impl Question<'_, '_, '_, '_, '_> {
    pub(crate) fn ask<B: Backend>(
        mut self,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::Events,
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
            QuestionKind::List(l) => l.ask(message, answers, b, events)?,
            QuestionKind::Rawlist(r) => r.ask(message, answers, b, events)?,
            QuestionKind::Expand(e) => e.ask(message, answers, b, events)?,
            QuestionKind::Checkbox(c) => c.ask(message, answers, b, events)?,
            QuestionKind::Password(p) => p.ask(message, answers, b, events)?,
            QuestionKind::Editor(e) => e.ask(message, answers, b, events)?,
            QuestionKind::Plugin(ref mut o) => o.ask(message, answers, b, events)?,
        };

        Ok(Some((name, res)))
    }

    crate::cfg_async! {
    pub(crate) async fn ask_async<B: Backend>(
        mut self,
        answers: &Answers,
        b: &mut B,
        events: &mut ui::events::AsyncEvents,
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
            QuestionKind::Input(i) => i.ask_async(message, answers, b, events).await?,
            QuestionKind::Int(i) => i.ask_async(message, answers, b, events).await?,
            QuestionKind::Float(f) => f.ask_async(message, answers, b, events).await?,
            QuestionKind::Confirm(c) => c.ask_async(message, answers, b, events).await?,
            QuestionKind::List(l) => l.ask_async(message, answers, b, events).await?,
            QuestionKind::Rawlist(r) => r.ask_async(message, answers, b, events).await?,
            QuestionKind::Expand(e) => e.ask_async(message, answers, b, events).await?,
            QuestionKind::Checkbox(c) => c.ask_async(message, answers, b, events).await?,
            QuestionKind::Password(p) => p.ask_async(message, answers, b, events).await?,
            QuestionKind::Editor(e) => e.ask_async(message, answers, b, events).await?,
            QuestionKind::Plugin(ref mut o) => o.ask_async(message, answers, b, events).await?,
        };

        Ok(Some((name, res)))
    }
    }
}

pub(crate) type BoxFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

macro_rules! handler {
    ($name:ident, $fn_trait:ident ( $($type:ty),* ) -> $return:ty) => {
        pub enum $name<'a, T> {
            Async(Box<dyn $fn_trait( $($type),* ) -> BoxFuture<'a, $return> + Send + Sync + 'a>),
            Sync(Box<dyn $fn_trait( $($type),* ) -> $return + Send + Sync + 'a>),
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
                    Self::Async(_) => f.write_str("Async(_)"),
                    Self::Sync(_) => f.write_str("Sync(_)"),
                    Self::None => f.write_str("None"),
                }
            }
        }
    };

    // The type signature of the function must only contain &T
    ($name:ident, unsafe ?Sized $fn_trait:ident ( $($type:ty),* ) -> $return:ty) => {
        pub enum $name<'a, T: ?Sized> {
            Async(Box<dyn $fn_trait( $($type),* ) -> BoxFuture<'a, $return> + Send + Sync + 'a>),
            Sync(Box<dyn $fn_trait( $($type),* ) -> $return + Send + Sync + 'a>),
            None,
        }

        // SAFETY: The type signature only contains &T as guaranteed by the invoker
        unsafe impl<'a, T: ?Sized> Send for $name<'a, T> where for<'b> &'b T: Send {}
        unsafe impl<'a, T: ?Sized> Sync for $name<'a, T> where for<'b> &'b T: Sync {}

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
                    Self::Async(_) => f.write_str("Async(_)"),
                    Self::Sync(_) => f.write_str("Sync(_)"),
                    Self::None => f.write_str("None"),
                }
            }
        }
    };
}

handler!(Filter, FnOnce(T, &Answers) -> T);
// SAFETY: The type signature only contains &T
handler!(Validate, unsafe ?Sized Fn(&T, &Answers) -> Result<(), String>);
handler!(ValidateByVal, Fn(T, &Answers) -> Result<(), String>);
// SAFETY: The type signature only contains &T
handler!(
    Transformer, unsafe ?Sized
    FnOnce(&T, &Answers, &mut dyn Backend) -> error::Result<()>
);
handler!(
    TransformerByVal,
    FnOnce(T, &Answers, &mut dyn Backend) -> error::Result<()>
);

#[doc(hidden)]
#[macro_export]
macro_rules! impl_filter_builder {
    // Unwieldy macro magic -- matches over lifetimes
    ($ty:ident < $( $pre_lifetime:lifetime ),*, f $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $filter:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 'f, $($post_lifetime),*> $ty<$($pre_lifetime),*, 'f, $($post_lifetime),*> {
            pub fn filter<'a, F>(self, filter: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce($t, &crate::Answers) -> $t + Send + Sync + 'a,
            {
                let $self = self;
                let $filter = crate::question::Filter::Sync(Box::new(filter));
                $body
            }

            pub fn filter_async<'a, F>(self, filter: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce($t, &crate::Answers) -> std::pin::Pin<Box<dyn std::future::Future<Output = $t> + Send + Sync + 'a>> + Send + Sync + 'a,
            {
                let $self = self;
                let $filter = crate::question::Filter::Async(Box::new(filter));
                $body
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_validate_builder {
    // Unwieldy macro magic -- matches over lifetimes
    ($ty:ident < $( $pre_lifetime:lifetime ),*, v $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $validate:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 'v, $($post_lifetime),*> $ty<$($pre_lifetime),*, 'v, $($post_lifetime),*> {
            pub fn validate<'a, F>(self, validate: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: Fn(&$t, &crate::Answers) -> Result<(), String> + Send + Sync + 'a,
            {
                let $self = self;
                let $validate = crate::question::Validate::Sync(Box::new(validate));
                $body
            }

            pub fn validate_async<'a, F>(self, validate: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: Fn(&$t, &crate::Answers) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + Sync + 'a>> + Send + Sync + 'a,
            {
                let $self = self;
                let $validate = crate::question::Validate::Async(Box::new(validate));
                $body
            }
        }
    };

    // Unwieldy macro magic -- matches over lifetimes
    (by val $ty:ident < $( $pre_lifetime:lifetime ),*, v $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $validate:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 'v, $($post_lifetime),*> $ty<$($pre_lifetime),*, 'v, $($post_lifetime),*> {
            pub fn validate<'a, F>(self, validate: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: Fn($t, &crate::Answers) -> Result<(), String> + Send + Sync + 'a,
            {
                let $self = self;
                let $validate = crate::question::ValidateByVal::Sync(Box::new(validate));
                $body
            }

            pub fn validate_async<'a, F>(self, validate: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: Fn($t, &crate::Answers) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + Sync + 'a>> + Send + Sync + 'a,
            {
                let $self = self;
                let $validate = crate::question::ValidateByVal::Async(Box::new(validate));
                $body
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_transformer_builder {
    // Unwieldy macro magic -- matches over lifetimes
    ($ty:ident < $( $pre_lifetime:lifetime ),*, t $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $transformer:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 't, $($post_lifetime),*> $ty<$($pre_lifetime),*, 't, $($post_lifetime),*> {
            pub fn transformer<'a, F>(self, transformer: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce(&$t, &crate::Answers, &mut dyn Backend) -> ui::error::Result<()> + Send + Sync + 'a,
            {
                let $self = self;
                let $transformer = crate::question::Transformer::Sync(Box::new(transformer));
                $body
            }

            pub fn transformer_async<'a, F>(self, transformer: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce(&$t, &crate::Answers, &mut dyn Backend) -> std::pin::Pin<Box<dyn std::future::Future<Output = ui::error::Result<()>> + Send + Sync + 'a>> + Send + Sync + 'a,
            {
                let $self = self;
                let $transformer = crate::question::Transformer::Async(Box::new(transformer));
                $body
            }
        }
    };

    // Unwieldy macro magic -- matches over lifetimes
    (by val $ty:ident < $( $pre_lifetime:lifetime ),*, t $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $transformer:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 't, $($post_lifetime),*> $ty<$($pre_lifetime),*, 't, $($post_lifetime),*> {
            pub fn transformer<'a, F>(self, transformer: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce($t, &crate::Answers, &mut dyn Backend) -> ui::error::Result<()> + Send + Sync + 'a,
            {
                let $self = self;
                let $transformer = crate::question::TransformerByVal::Sync(Box::new(transformer));
                $body
            }

            pub fn transformer_async<'a, F>(self, transformer: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce($t, &crate::Answers, &mut dyn Backend) -> std::pin::Pin<Box<dyn std::future::Future<Output = ui::error::Result<()>> + Send + Sync + 'a>> + Send + Sync + 'a,
            {
                let $self = self;
                let $transformer = crate::question::TransformerByVal::Async(Box::new(transformer));
                $body
            }
        }
    }
}

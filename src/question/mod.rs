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
mod rawlist;

use crate::{error, Answer, Answers};
pub use choice::Choice;
use choice::{get_sep_str, ChoiceList};
use options::Options;

use std::io::prelude::*;

type Filter<'a, T> = dyn FnOnce(T, &Answers) -> T + 'a;
type Validate<'a, T> = dyn Fn(&T, &Answers) -> Result<(), String> + 'a;
type ValidateV<'a, T> = dyn Fn(T, &Answers) -> Result<(), String> + 'a;
type Transformer<'a, T> = dyn FnOnce(&T, &Answers, &mut dyn Write) -> error::Result<()> + 'a;
type TransformerV<'a, T> = dyn FnOnce(T, &Answers, &mut dyn Write) -> error::Result<()> + 'a;

fn some<T>(_: T) -> &'static str {
    "Some(_)"
}

fn none() -> &'static str {
    "None"
}

#[derive(Debug)]
pub struct Question<'m, 'w, 'f, 'v, 't> {
    kind: QuestionKind<'f, 'v, 't>,
    opts: Options<'m, 'w>,
}

impl<'m, 'w, 'f, 'v, 't> Question<'m, 'w, 'f, 'v, 't> {
    fn new(opts: Options<'m, 'w>, kind: QuestionKind<'f, 'v, 't>) -> Self {
        Self { opts, kind }
    }
}

#[derive(Debug)]
enum QuestionKind<'f, 'v, 't> {
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
}

impl Question<'_, '_, '_, '_, '_> {
    pub fn ask<W: Write>(
        self,
        answers: &Answers,
        w: &mut W,
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
            QuestionKind::Input(i) => i.ask(message, answers, w)?,
            QuestionKind::Int(i) => i.ask(message, answers, w)?,
            QuestionKind::Float(f) => f.ask(message, answers, w)?,
            QuestionKind::Confirm(c) => c.ask(message, answers, w)?,
            QuestionKind::List(l) => l.ask(message, answers, w)?,
            QuestionKind::Rawlist(r) => r.ask(message, answers, w)?,
            QuestionKind::Expand(e) => e.ask(message, answers, w)?,
            QuestionKind::Checkbox(c) => c.ask(message, answers, w)?,
            QuestionKind::Password(p) => p.ask(message, answers, w)?,
            QuestionKind::Editor(e) => e.ask(message, answers, w)?,
        };

        Ok(Some((name, res)))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_filter_builder {
    // Unwieldy macro magic -- matches over lifetimes
    ($ty:ident < $( $pre_lifetime:lifetime ),*, f $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $filter:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 'f, $($post_lifetime),*> $ty<$($pre_lifetime),*, 'f, $($post_lifetime),*> {
            pub fn filter<'a, F>(self, filter: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce($t, &crate::Answers) -> $t + 'a,
            {
                let $self = self;
                let $filter: Option<Box<crate::question::Filter<'a, $t>>> = Some(Box::new(filter));
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
                F: Fn(&$t, &crate::Answers) -> Result<(), String> + 'a,
            {
                let $self = self;
                let $validate: Option<Box<crate::question::Validate<'a, $t>>> = Some(Box::new(validate));
                $body
            }
        }
    };

    // Unwieldy macro magic -- matches over lifetimes
    (by val $ty:ident < $( $pre_lifetime:lifetime ),*, v $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $validate:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 'v, $($post_lifetime),*> $ty<$($pre_lifetime),*, 'v, $($post_lifetime),*> {
            pub fn validate<'a, F>(self, validate: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: Fn($t, &crate::Answers) -> Result<(), String> + 'a,
            {
                let $self = self;
                let $validate: Option<Box<crate::question::ValidateV<'a, $t>>> = Some(Box::new(validate));
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
                F: FnOnce(&$t, &crate::Answers, &mut dyn std::io::Write) -> crate::error::Result<()> + 'a,
            {
                let $self = self;
                let $transformer: Option<Box<crate::question::Transformer<'a, $t>>> = Some(Box::new(transformer));
                $body
            }
        }
    };

    // Unwieldy macro magic -- matches over lifetimes
    (by val $ty:ident < $( $pre_lifetime:lifetime ),*, t $(,)? $( $post_lifetime:lifetime ),* > $t:ty; ($self:ident, $transformer:ident) => $body:expr) => {
        impl<$($pre_lifetime),*, 't, $($post_lifetime),*> $ty<$($pre_lifetime),*, 't, $($post_lifetime),*> {
            pub fn transformer<'a, F>(self, transformer: F) -> $ty<$($pre_lifetime),*, 'a, $($post_lifetime),*>
            where
                F: FnOnce($t, &crate::Answers, &mut dyn std::io::Write) -> crate::error::Result<()> + 'a,
            {
                let $self = self;
                let $transformer: Option<Box<crate::question::TransformerV<'a, $t>>> = Some(Box::new(transformer));
                $body
            }
        }
    }
}

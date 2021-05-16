use ui::{
    backend::{Backend, Color},
    error,
    events::KeyEvent,
    widgets, Prompt, Validation, Widget,
};

use super::{
    Filter, Options, TransformByVal as Transform, ValidateByVal as Validate,
};
use crate::{Answer, Answers};

#[derive(Debug, Default)]
pub struct Float<'f, 'v, 't> {
    default: Option<f64>,
    filter: Filter<'f, f64>,
    validate: Validate<'v, f64>,
    transform: Transform<'t, f64>,
}

#[derive(Debug, Default)]
pub struct Int<'f, 'v, 't> {
    default: Option<i64>,
    filter: Filter<'f, i64>,
    validate: Validate<'v, i64>,
    transform: Transform<'t, i64>,
}

impl Int<'_, '_, '_> {
    fn write<B: Backend>(i: i64, b: &mut B) -> error::Result<()> {
        b.set_fg(Color::Cyan)?;
        write!(b, "{}", i)?;
        b.set_fg(Color::Reset)?;
        b.write_all(b"\n")?;
        b.flush().map_err(Into::into)
    }

    fn filter_map_char(c: char) -> Option<char> {
        if c.is_digit(10) || c == '-' || c == '+' {
            Some(c)
        } else {
            None
        }
    }
}

impl Float<'_, '_, '_> {
    fn write<B: Backend>(f: f64, b: &mut B) -> error::Result<()> {
        b.set_fg(Color::Cyan)?;
        if f.log10().abs() > 19.0 {
            write!(b, "{:e}", f)?;
        } else {
            write!(b, "{}", f)?;
        }
        b.set_fg(Color::Reset)?;
        b.write_all(b"\n")?;
        b.flush().map_err(Into::into)
    }

    fn filter_map_char(c: char) -> Option<char> {
        if Int::filter_map_char(c).is_some() || c == '.' || c == 'e' || c == 'E' {
            Some(c)
        } else {
            None
        }
    }
}

macro_rules! impl_number_prompt {
    ($prompt_name:ident, $type:ident, $inner_ty:ty) => {
        struct $prompt_name<'f, 'v, 't, 'a> {
            number: $type<'f, 'v, 't>,
            message: String,
            input: widgets::StringInput,
            hint: Option<String>,
            answers: &'a Answers,
        }

        impl $prompt_name<'_, '_, '_, '_> {
            fn parse(&self) -> Result<$inner_ty, String> {
                self.input.value().parse::<$inner_ty>().map_err(|e| e.to_string())
            }
        }

        impl Widget for $prompt_name<'_, '_, '_, '_> {
            fn render<B: Backend>(
                &mut self,
                max_width: usize,
                b: &mut B,
            ) -> error::Result<()> {
                self.input.render(max_width, b)
            }

            fn height(&self) -> usize {
                self.input.height()
            }

            fn handle_key(&mut self, key: KeyEvent) -> bool {
                self.input.handle_key(key)
            }

            fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
                self.input.cursor_pos(prompt_len)
            }
        }

        impl Prompt for $prompt_name<'_, '_, '_, '_> {
            type ValidateErr = String;
            type Output = $inner_ty;

            fn prompt(&self) -> &str {
                &self.message
            }

            fn hint(&self) -> Option<&str> {
                self.hint.as_ref().map(String::as_ref)
            }

            fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
                if self.input.value().is_empty() && self.has_default() {
                    return Ok(Validation::Finish);
                }
                let n = self.parse()?;

                if let Validate::Sync(ref validate) = self.number.validate {
                    validate(n, self.answers)?;
                }

                Ok(Validation::Finish)
            }

            fn finish(self) -> Self::Output {
                if self.input.value().is_empty() && self.has_default() {
                    return self.number.default.unwrap();
                }

                let n = self.parse().unwrap();
                match self.number.filter {
                    Filter::Sync(filter) => filter(n, self.answers),
                    _ => n,
                }
            }

            fn has_default(&self) -> bool {
                self.number.default.is_some()
            }
            fn finish_default(self) -> Self::Output {
                self.number.default.unwrap()
            }
        }

        crate::cfg_async! {
        #[async_trait::async_trait]
        impl ui::AsyncPrompt for $prompt_name<'_, '_, '_, '_> {
            async fn finish_async(self) -> Self::Output {
                if self.input.value().is_empty() && self.has_default() {
                    return self.number.default.unwrap();
                }

                let n = self.parse().unwrap();
                match self.number.filter {
                    Filter::Async(filter) => filter(n, self.answers).await,
                    Filter::Sync(filter) => filter(n, self.answers),
                    Filter::None => n,
                }
            }

            fn try_validate_sync(&mut self) -> Option<Result<Validation, Self::ValidateErr>> {
                if self.input.value().is_empty() && self.has_default() {
                    return Some(Ok(Validation::Finish));
                }

                let n = match self.parse() {
                    Ok(n) => n,
                    Err(e) => return Some(Err(e)),
                };


                match self.number.validate {
                    Validate::Sync(ref validate) => {
                        Some(validate(n, self.answers).map(|_| Validation::Finish))
                    }
                    _ => None,
                }
            }

            async fn validate_async(&mut self) -> Result<Validation, Self::ValidateErr> {
                if let Validate::Async(ref validate) = self.number.validate {
                    validate(self.parse().unwrap(), self.answers).await?;
                }

                Ok(Validation::Finish)
            }
        }
        }
    };
}

impl_number_prompt!(IntPrompt, Int, i64);
impl_number_prompt!(FloatPrompt, Float, f64);

macro_rules! impl_ask {
    ($t:ident, $prompt_name:ident) => {
        impl $t<'_, '_, '_> {
            pub(crate) fn ask<B: Backend>(
                mut self,
                message: String,
                answers: &Answers,
                b: &mut B,
                events: &mut ui::events::Events,
            ) -> error::Result<Answer> {
                let transform = self.transform.take();

                let ans = ui::Input::new(
                    $prompt_name {
                        hint: self.default.map(|default| format!("({})", default)),
                        input: widgets::StringInput::new(Self::filter_map_char),
                        number: self,
                        message,
                        answers,
                    },
                    b,
                )
                .run(events)?;

                match transform {
                    Transform::Sync(transform) => transform(ans, answers, b)?,
                    _ => Self::write(ans, b)?,
                }

                Ok(Answer::$t(ans))
            }

            crate::cfg_async! {
            pub(crate) async fn ask_async<B: Backend>(
                mut self,
                message: String,
                answers: &Answers,
                b: &mut B,
                events: &mut ui::events::AsyncEvents,
            ) -> error::Result<Answer> {
                let transform = self.transform.take();

                let ans = ui::Input::new($prompt_name {
                    hint: self.default.map(|default| format!("({})", default)),
                    input: widgets::StringInput::new(Self::filter_map_char),
                    number: self,
                    message,
                    answers,
                }, b)
                .run_async(events)
                .await?;

                match transform {
                    Transform::Async(transform) => transform(ans, answers, b).await?,
                    Transform::Sync(transform) => transform(ans, answers, b)?,
                    _ => Self::write(ans, b)?,
                }

                Ok(Answer::$t(ans))
            }
            }
        }
    };
}

impl_ask!(Int, IntPrompt);
impl_ask!(Float, FloatPrompt);

macro_rules! builder {
    ($builder_name:ident, $type:ident, $inner_ty:ty, $kind:expr) => {
        pub struct $builder_name<'m, 'w, 'f, 'v, 't> {
            opts: Options<'m, 'w>,
            inner: $type<'f, 'v, 't>,
        }

        impl<'m, 'w, 'f, 'v, 't> $builder_name<'m, 'w, 'f, 'v, 't> {
            pub(crate) fn new(name: String) -> Self {
                $builder_name {
                    opts: Options::new(name),
                    inner: Default::default(),
                }
            }

            pub fn default(mut self, default: $inner_ty) -> Self {
                self.inner.default = Some(default);
                self
            }

            pub fn build(self) -> super::Question<'m, 'w, 'f, 'v, 't> {
                super::Question::new(self.opts, $kind(self.inner))
            }
        }

        impl<'m, 'w, 'f, 'v, 't> From<$builder_name<'m, 'w, 'f, 'v, 't>> for super::Question<'m, 'w, 'f, 'v, 't> {
            fn from(builder: $builder_name<'m, 'w, 'f, 'v, 't>) -> Self {
                builder.build()
            }
        }

        crate::impl_options_builder!($builder_name<'f, 'v, 't>; (this, opts) => {
            $builder_name {
                opts,
                inner: this.inner,
            }
        });

        crate::impl_filter_builder!($builder_name<'m, 'w, f, 'v, 't> $inner_ty; (this, filter) => {
            $builder_name {
                opts: this.opts,
                inner: $type {
                    filter,
                    default: this.inner.default,
                    validate: this.inner.validate,
                    transform: this.inner.transform,
                }
            }
        });

        crate::impl_validate_builder!(by val $builder_name<'m, 'w, 'f, v, 't> $inner_ty; (this, validate) => {
            $builder_name {
                opts: this.opts,
                inner: $type {
                    validate,
                    default: this.inner.default,
                    filter: this.inner.filter,
                    transform: this.inner.transform,
                }
            }
        });

        crate::impl_transform_builder!(by val $builder_name<'m, 'w, 'f, 'v, t> $inner_ty; (this, transform) => {
            $builder_name {
                opts: this.opts,
                inner: $type {
                    transform,
                    validate: this.inner.validate,
                    default: this.inner.default,
                    filter: this.inner.filter,
                }
            }
        });
    };
}

builder!(IntBuilder, Int, i64, super::QuestionKind::Int);
builder!(FloatBuilder, Float, f64, super::QuestionKind::Float);

use std::fmt;

use crossterm::style::{Color, ResetColor, SetForegroundColor};
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::{none, some, Filter, Options, TransformerV, ValidateV};

#[derive(Default)]
pub struct Float<'f, 'v, 't> {
    default: Option<f64>,
    filter: Option<Box<Filter<'f, f64>>>,
    validate: Option<Box<ValidateV<'v, f64>>>,
    transformer: Option<Box<TransformerV<'t, f64>>>,
}

impl fmt::Debug for Float<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Float")
            .field("default", &self.default)
            .field("filter", &self.filter.as_ref().map_or_else(none, some))
            .field("validate", &self.validate.as_ref().map_or_else(none, some))
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
}

#[derive(Default)]
pub struct Int<'f, 'v, 't> {
    default: Option<i64>,
    filter: Option<Box<Filter<'f, i64>>>,
    validate: Option<Box<ValidateV<'v, i64>>>,
    transformer: Option<Box<TransformerV<'t, i64>>>,
}

impl fmt::Debug for Int<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Int")
            .field("default", &self.default)
            .field("filter", &self.filter.as_ref().map_or_else(none, some))
            .field("validate", &self.validate.as_ref().map_or_else(none, some))
            .field(
                "transformer",
                &self.transformer.as_ref().map_or_else(none, some),
            )
            .finish()
    }
}

trait Number {
    type Inner;

    fn validate(&self, inner: Self::Inner) -> Result<(), String>;
    fn filter(self, inner: Self::Inner) -> Self::Inner;
    fn filter_map_char(c: char) -> Option<char>;
    fn parse(s: &str) -> Result<Self::Inner, String>;
    fn default(&self) -> Option<Self::Inner>;
    fn write<W: std::io::Write>(inner: Self::Inner, w: &mut W) -> error::Result<()>;
    fn finish(inner: Self::Inner) -> Answer;
}

impl Number for Int<'_, '_, '_> {
    type Inner = i64;

    fn filter_map_char(c: char) -> Option<char> {
        if c.is_digit(10) || c == '-' || c == '+' {
            Some(c)
        } else {
            None
        }
    }

    fn parse(s: &str) -> Result<i64, String> {
        s.parse::<i64>().map_err(|e| e.to_string())
    }

    fn default(&self) -> Option<Self::Inner> {
        self.default
    }

    fn validate(&self, i: Self::Inner) -> Result<(), String> {
        if let Some(ref validate) = self.validate {
            validate(i)
        } else {
            Ok(())
        }
    }

    fn filter(self, i: Self::Inner) -> Self::Inner {
        if let Some(filter) = self.filter {
            filter(i)
        } else {
            i
        }
    }

    fn write<W: std::io::Write>(i: Self::Inner, w: &mut W) -> error::Result<()> {
        writeln!(
            w,
            "{}{}{}",
            SetForegroundColor(Color::DarkCyan),
            i,
            ResetColor,
        )
        .map_err(Into::into)
    }

    fn finish(i: Self::Inner) -> Answer {
        Answer::Int(i)
    }
}
impl Number for Float<'_, '_, '_> {
    type Inner = f64;

    fn filter_map_char(c: char) -> Option<char> {
        if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
            Some(c)
        } else {
            None
        }
    }

    fn parse(s: &str) -> Result<f64, String> {
        s.parse::<f64>().map_err(|e| e.to_string())
    }

    fn default(&self) -> Option<Self::Inner> {
        self.default
    }

    fn validate(&self, f: Self::Inner) -> Result<(), String> {
        if let Some(ref validate) = self.validate {
            validate(f)
        } else {
            Ok(())
        }
    }

    fn filter(self, f: Self::Inner) -> Self::Inner {
        if let Some(filter) = self.filter {
            filter(f)
        } else {
            f
        }
    }

    fn write<W: std::io::Write>(f: Self::Inner, w: &mut W) -> error::Result<()> {
        write!(w, "{}", SetForegroundColor(Color::DarkCyan))?;
        if f.log10().abs() > 19.0 {
            write!(w, "{:e}", f)?;
        } else {
            write!(w, "{}", f)?;
        }
        writeln!(w, "{}", ResetColor).map_err(Into::into)
    }

    fn finish(f: Self::Inner) -> Answer {
        Answer::Float(f)
    }
}

struct NumberPrompt<N> {
    number: N,
    message: String,
    input: widgets::StringInput,
    hint: Option<String>,
}

impl<N: Number> Widget for NumberPrompt<N> {
    fn render<W: std::io::Write>(&mut self, max_width: usize, w: &mut W) -> crossterm::Result<()> {
        self.input.render(max_width, w)
    }

    fn height(&self) -> usize {
        self.input.height()
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.input.handle_key(key)
    }

    fn cursor_pos(&self, prompt_len: u16) -> (u16, u16) {
        self.input.cursor_pos(prompt_len)
    }
}

impl<N: Number> ui::Prompt for NumberPrompt<N> {
    type ValidateErr = String;
    type Output = N::Inner;

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

        self.number
            .validate(N::parse(self.input.value())?)
            .map(|_| Validation::Finish)
    }
    fn finish(self) -> Self::Output {
        if self.input.value().is_empty() && self.has_default() {
            return self.number.default().unwrap();
        }
        self.number.filter(N::parse(self.input.value()).unwrap())
    }

    fn has_default(&self) -> bool {
        self.number.default().is_some()
    }
    fn finish_default(self) -> Self::Output {
        self.number.default().unwrap()
    }
}

macro_rules! impl_ask {
    ($t:ty) => {
        impl $t {
            pub(crate) fn ask<W: std::io::Write>(
                mut self,
                message: String,
                w: &mut W,
            ) -> error::Result<Answer> {
                let transformer = self.transformer.take();

                let ans = ui::Input::new(NumberPrompt {
                    hint: self.default.map(|default| format!("({})", default)),
                    input: widgets::StringInput::new(Self::filter_map_char),
                    number: self,
                    message,
                })
                .run(w)?;

                match transformer {
                    Some(transformer) => transformer(ans, w)?,
                    None => Self::write(ans, w)?,
                }

                Ok(Self::finish(ans))
            }
        }
    };
}

impl_ask!(Int<'_, '_, '_>);
impl_ask!(Float<'_, '_, '_>);

macro_rules! builder {
    ($builder_name:ident, $type:ident, $inner_ty:ty, $kind:expr) => {
        pub struct $builder_name<'m, 'w, 'f, 'v, 't> {
            opts: Options<'m, 'w>,
            inner: $type<'f, 'v, 't>,
        }

        impl<'m, 'w, 'f, 'v, 't> $builder_name<'m, 'w, 'f, 'v, 't> {
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
                    transformer: this.inner.transformer,
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
                    transformer: this.inner.transformer,
                }
            }
        });

        crate::impl_transformer_builder!(by val $builder_name<'m, 'w, 'f, 'v, t> $inner_ty; (this, transformer) => {
            $builder_name {
                opts: this.opts,
                inner: $type {
                    transformer,
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

impl super::Question<'static, 'static, 'static, 'static, 'static> {
    pub fn int<N: Into<String>>(
        name: N,
    ) -> IntBuilder<'static, 'static, 'static, 'static, 'static> {
        IntBuilder {
            opts: Options::new(name.into()),
            inner: Default::default(),
        }
    }

    pub fn float<N: Into<String>>(
        name: N,
    ) -> FloatBuilder<'static, 'static, 'static, 'static, 'static> {
        FloatBuilder {
            opts: Options::new(name.into()),
            inner: Default::default(),
        }
    }
}

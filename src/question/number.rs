use crossterm::style::{Color, ResetColor, SetForegroundColor};
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::Options;

#[derive(Default, Debug)]
pub struct Float {
    default: Option<f64>,
}

#[derive(Default, Debug)]
pub struct Int {
    default: Option<i64>,
}

trait Number {
    type Inner;

    fn filter_map_char(c: char) -> Option<char>;
    fn parse(s: &str) -> Result<Self::Inner, String>;
    fn default(&self) -> Option<Self::Inner>;
    fn finish<W: std::io::Write>(inner: Self::Inner, w: &mut W) -> error::Result<Answer>;
}

impl Number for Int {
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

    fn finish<W: std::io::Write>(i: Self::Inner, w: &mut W) -> error::Result<Answer> {
        writeln!(
            w,
            "{}{}{}",
            SetForegroundColor(Color::DarkCyan),
            i,
            ResetColor,
        )?;

        Ok(Answer::Int(i))
    }
}
impl Number for Float {
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

    fn finish<W: std::io::Write>(f: Self::Inner, w: &mut W) -> error::Result<Answer> {
        write!(w, "{}", SetForegroundColor(Color::DarkCyan))?;
        if f.log10().abs() > 19.0 {
            write!(w, "{:e}", f)?;
        } else {
            write!(w, "{}", f)?;
        }
        writeln!(w, "{}", ResetColor)?;

        Ok(Answer::Float(f))
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
        N::parse(self.input.value()).map(|_| Validation::Finish)
    }
    fn finish(self) -> Self::Output {
        if self.input.value().is_empty() && self.has_default() {
            return self.number.default().unwrap();
        }
        N::parse(self.input.value()).unwrap()
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
                self,
                message: String,
                w: &mut W,
            ) -> error::Result<Answer> {
                let ans = ui::Input::new(NumberPrompt {
                    hint: self.default.map(|default| format!("({})", default)),
                    input: widgets::StringInput::new(Self::filter_map_char),
                    number: self,
                    message,
                })
                .run(w)?;

                Self::finish(ans, w)
            }
        }
    };
}

impl_ask!(Int);
impl_ask!(Float);

macro_rules! builder {
    ($builder_name:ident, $type:ty, $inner_ty:ty, $kind:expr) => {
        pub struct $builder_name<'m, 'w> {
            opts: Options<'m, 'w>,
            inner: $type,
        }

        impl<'m, 'w> $builder_name<'m, 'w> {
            pub fn default(mut self, default: $inner_ty) -> Self {
                self.inner.default = Some(default);
                self
            }

            pub fn build(self) -> super::Question<'m, 'w> {
                super::Question::new(self.opts, $kind(self.inner))
            }
        }

        impl<'m, 'w> From<$builder_name<'m, 'w>> for super::Question<'m, 'w> {
            fn from(builder: $builder_name<'m, 'w>) -> Self {
                builder.build()
            }
        }

        crate::impl_options_builder!($builder_name; (this, opts) => {
            $builder_name {
                opts,
                inner: this.inner,
            }
        });
    };
}

builder!(IntBuilder, Int, i64, super::QuestionKind::Int);
builder!(FloatBuilder, Float, f64, super::QuestionKind::Float);

impl super::Question<'static, 'static> {
    pub fn int<N: Into<String>>(name: N) -> IntBuilder<'static, 'static> {
        IntBuilder {
            opts: Options::new(name.into()),
            inner: Default::default(),
        }
    }

    pub fn float<N: Into<String>>(name: N) -> FloatBuilder<'static, 'static> {
        FloatBuilder {
            opts: Options::new(name.into()),
            inner: Default::default(),
        }
    }
}

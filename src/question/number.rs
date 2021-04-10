use crossterm::style::{Color, ResetColor, SetForegroundColor};
use ui::{widgets, Validation, Widget};

use crate::{error, Answer};

use super::Options;

pub struct Float {
    default: f64,
}

pub struct Int {
    default: i64,
}

trait Number {
    type Inner;

    fn filter_map_char(c: char) -> Option<char>;
    fn parse(s: &str) -> Result<Self::Inner, String>;
    fn default(&self) -> Self::Inner;
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

    fn default(&self) -> Self::Inner {
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

    fn default(&self) -> Self::Inner {
        self.default
    }

    fn finish<W: std::io::Write>(f: Self::Inner, w: &mut W) -> error::Result<Answer> {
        write!(w, "{}", SetForegroundColor(Color::DarkCyan))?;
        if f > 1e20 {
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
    opts: Options,
    input: widgets::StringInput,
    hint: String,
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
        &self.opts.message
    }

    fn hint(&self) -> Option<&str> {
        Some(&self.hint)
    }

    fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
        N::parse(self.input.value()).map(|_| Validation::Finish)
    }

    fn finish(self) -> Self::Output {
        N::parse(self.input.value()).unwrap_or_else(|_| self.number.default())
    }
    fn finish_default(self) -> Self::Output {
        self.number.default()
    }
}

macro_rules! impl_ask {
    ($t:ty) => {
        impl $t {
            pub(crate) fn ask<W: std::io::Write>(
                self,
                opts: super::Options,
                w: &mut W,
            ) -> error::Result<Answer> {
                let ans = ui::Input::new(NumberPrompt {
                    hint: format!("({})", self.default),
                    input: widgets::StringInput::new(Self::filter_map_char),
                    number: self,
                    opts,
                })
                .run(w)?;

                Self::finish(ans, w)
            }
        }
    };
}

impl_ask!(Int);
impl_ask!(Float);

impl super::Question {
    pub fn int(name: String, message: String, default: i64) -> Self {
        Self::new(name, message, super::QuestionKind::Int(Int { default }))
    }

    pub fn float(name: String, message: String, default: f64) -> Self {
        Self::new(name, message, super::QuestionKind::Float(Float { default }))
    }
}

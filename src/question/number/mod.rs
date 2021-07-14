use std::{fmt::Write, io};

use ui::{
    backend::Backend,
    events::{EventIterator, KeyCode, KeyEvent},
    style::Color,
    widgets, Prompt, Validation, Widget,
};

use super::{Filter, TransformByVal as Transform, ValidateByVal as Validate};
use crate::{Answer, Answers};

// This is not actually unreachable, it is re-exported in crate::question
#[allow(unreachable_pub)]
pub use builder::{FloatBuilder, IntBuilder};

mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct Float<'a> {
    default: Option<f64>,
    filter: Filter<'a, f64>,
    validate: Validate<'a, f64>,
    transform: Transform<'a, f64>,
}

#[derive(Debug, Default)]
pub(super) struct Int<'a> {
    default: Option<i64>,
    filter: Filter<'a, i64>,
    validate: Validate<'a, i64>,
    transform: Transform<'a, i64>,
}

impl Int<'_> {
    fn write<B: Backend>(i: i64, b: &mut B) -> io::Result<()> {
        b.set_fg(Color::Cyan)?;
        write!(b, "{}", i)?;
        b.set_fg(Color::Reset)
    }

    fn delta(i: i64, delta: i64) -> i64 {
        i.wrapping_add(delta)
    }

    fn filter_map(c: char) -> Option<char> {
        if c.is_digit(10) || c == '-' || c == '+' {
            Some(c)
        } else {
            None
        }
    }
}

impl Float<'_> {
    fn write<B: Backend>(f: f64, b: &mut B) -> io::Result<()> {
        b.set_fg(Color::Cyan)?;
        if f.log10().abs() > 19.0 {
            write!(b, "{:e}", f)?;
        } else {
            write!(b, "{}", f)?;
        }
        b.set_fg(Color::Reset)
    }

    fn delta(f: f64, delta: i64) -> f64 {
        f + delta as f64
    }

    fn filter_map(c: char) -> Option<char> {
        if Int::filter_map(c).is_some() || ['.', 'e', 'E', 'i', 'n', 'f'].contains(&c) {
            Some(c)
        } else {
            None
        }
    }
}

macro_rules! impl_number_prompt {
    ($prompt_name:ident, $type:ident, $inner_ty:ty) => {
        struct $prompt_name<'n, 'a> {
            prompt: widgets::Prompt<&'a str, String>,
            number: $type<'n>,
            input: widgets::StringInput,
            answers: &'a Answers,
        }

        impl $prompt_name<'_, '_> {
            fn parse(&self) -> Result<$inner_ty, String> {
                self.input
                    .value()
                    .parse::<$inner_ty>()
                    .map_err(|e| e.to_string())
            }
        }

        impl Widget for $prompt_name<'_, '_> {
            fn render<B: Backend>(
                &mut self,
                layout: &mut ui::layout::Layout,
                b: &mut B,
            ) -> io::Result<()> {
                self.prompt.render(layout, b)?;
                self.input.render(layout, b)
            }

            fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
                self.prompt.height(layout) + self.input.height(layout) - 1
            }

            fn handle_key(&mut self, key: KeyEvent) -> bool {
                if self.input.handle_key(key) {
                    return true;
                }

                let n = match (key.code, self.parse()) {
                    (KeyCode::PageUp, Ok(n)) => $type::delta(n, 10),
                    (KeyCode::PageDown, Ok(n)) => $type::delta(n, -10),
                    (KeyCode::Up, Ok(n)) => $type::delta(n, 1),
                    (KeyCode::Down, Ok(n)) => $type::delta(n, -1),
                    _ => return false,
                };

                self.input.replace_with(|mut s| {
                    s.clear();
                    write!(s, "{}", n).expect("Failed to write number to the string");
                    s
                });
                true
            }

            fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
                self.input
                    .cursor_pos(layout.with_cursor_pos(self.prompt.cursor_pos(layout)))
            }
        }

        impl Prompt for $prompt_name<'_, '_> {
            type ValidateErr = widgets::Text<String>;
            type Output = $inner_ty;

            fn validate(&mut self) -> Result<Validation, Self::ValidateErr> {
                if self.input.value().is_empty() && self.number.default.is_some() {
                    return Ok(Validation::Finish);
                }
                let n = self.parse()?;

                if let Validate::Sync(ref mut validate) = self.number.validate {
                    validate(n, self.answers)?;
                }

                Ok(Validation::Finish)
            }

            fn finish(self) -> Self::Output {
                let n = match self.number.default {
                    Some(default) if self.input.value().is_empty() => default,
                    _ => self
                        .parse()
                        .expect("Validation would fail if number cannot be parsed"),
                };

                match self.number.filter {
                    Filter::Sync(filter) => filter(n, self.answers),
                    _ => n,
                }
            }
        }
    };
}

impl_number_prompt!(IntPrompt, Int, i64);
impl_number_prompt!(FloatPrompt, Float, f64);

macro_rules! impl_ask {
    ($t:ident, $prompt_name:ident) => {
        impl<'n> $t<'n> {
            fn into_prompt<'a>(
                self,
                message: &'a str,
                answers: &'a Answers,
            ) -> $prompt_name<'n, 'a> {
                $prompt_name {
                    prompt: widgets::Prompt::new(message)
                        .with_optional_hint(self.default.as_ref().map(ToString::to_string)),
                    input: widgets::StringInput::with_filter_map(Self::filter_map),
                    number: self,
                    answers,
                }
            }

            pub(crate) fn ask<B: Backend, E: EventIterator>(
                mut self,
                message: String,
                answers: &Answers,
                b: &mut B,
                events: &mut E,
            ) -> ui::Result<Answer> {
                let transform = self.transform.take();

                let ans = ui::Input::new(self.into_prompt(&message, answers), b).run(events)?;

                crate::write_final!(transform, message, ans, answers, b, Self::write(ans, b)?);

                Ok(Answer::$t(ans))
            }
        }
    };
}

impl_ask!(Int, IntPrompt);
impl_ask!(Float, FloatPrompt);

use std::{fmt::Write, io};

use ui::{
    backend::Backend,
    events::{EventIterator, KeyCode, KeyEvent},
    style::Color,
    widgets, Prompt, Validation, Widget,
};

use super::{
    Filter, TransformByVal as Transform, ValidateByVal as Validate,
    ValidateOnKeyByVal as ValidateOnKey,
};
use crate::{Answer, Answers};

// This is not actually unreachable, it is re-exported in crate::question
#[allow(unreachable_pub)]
pub use builder::{FloatBuilder, IntBuilder};

mod builder;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct Float<'a> {
    default: Option<(f64, String)>,
    filter: Filter<'a, f64>,
    validate: Validate<'a, f64>,
    validate_on_key: ValidateOnKey<'a, f64>,
    transform: Transform<'a, f64>,
}

#[derive(Debug, Default)]
pub(super) struct Int<'a> {
    default: Option<(i64, String)>,
    filter: Filter<'a, i64>,
    validate: Validate<'a, i64>,
    validate_on_key: ValidateOnKey<'a, i64>,
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
        if c.is_ascii_digit() || c == '-' || c == '+' {
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
            is_valid: bool,
            answers: &'a Answers,
        }

        impl $prompt_name<'_, '_> {
            fn parse(&self) -> Result<$inner_ty, String> {
                self.input
                    .value()
                    .parse::<$inner_ty>()
                    .map_err(|e| e.to_string())
            }

            fn get_remaining_default(&self) -> Option<&str> {
                if let Some((_, ref default)) = self.number.default {
                    let input = self.input.value();
                    if default.starts_with(self.input.value()) {
                        return Some(&default[input.len()..]);
                    }
                }

                None
            }

            fn check_complete_default(&mut self) -> bool {
                if self.get_remaining_default().is_some() {
                    let default = &self.number.default.as_ref().unwrap().1;
                    self.input.set_value(default.clone());
                    self.input.set_at(default.len());
                    self.is_valid = true;

                    true
                } else {
                    false
                }
            }

            fn validate_on_key(&mut self, n: $inner_ty) {
                if let ValidateOnKey::Sync(ref mut validate) = self.number.validate_on_key {
                    self.is_valid = validate(n, self.answers);
                } else {
                    self.is_valid = true;
                }
            }
        }

        impl Widget for $prompt_name<'_, '_> {
            fn render<B: Backend>(
                &mut self,
                layout: &mut ui::layout::Layout,
                b: &mut B,
            ) -> io::Result<()> {
                let mut original_layout = *layout;
                self.prompt.render(layout, b)?;

                // if the current input does not satisfy the on key validation, then we show its wrong by
                // using the red colour
                if !self.is_valid {
                    b.set_fg(ui::style::Color::Red)?;
                }
                self.input.render(layout, b)?;
                if !self.is_valid {
                    b.set_fg(ui::style::Color::Reset)?;
                }

                if let Some(default) = self.get_remaining_default() {
                    b.set_fg(ui::style::Color::DarkGrey)?;
                    write!(b, "{}", default)?;
                    b.set_fg(ui::style::Color::Reset)?;
                    // We need to update the layout to reflect the rest of the hint that is
                    // rendered. Instead of doing the math to compute where the cursor ends after
                    // rendering, we use the height function which already calculates it.
                    self.height(&mut original_layout);
                    *layout = original_layout;
                }

                Ok(())
            }

            fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
                let mut height = self.prompt.height(layout) - 1;

                if self.get_remaining_default().is_some() {
                    let mut width = self.number.default.as_ref().unwrap().1.len() as u16;

                    if width > layout.line_width() {
                        width -= layout.line_width();

                        layout.line_offset = width % layout.width;
                        layout.offset_y += 1 + width / layout.width;

                        height += 2 + width / layout.width;
                    } else {
                        layout.line_offset += width;
                        height += 1;
                    }
                } else {
                    height = self.input.height(layout);
                }

                height
            }

            fn handle_key(&mut self, key: KeyEvent) -> bool {
                if self.input.handle_key(key) {
                    match self.parse() {
                        Ok(n) => self.validate_on_key(n),
                        Err(_) => self.is_valid = false,
                    }

                    return true;
                } else if key.code == KeyCode::Tab || key.code == KeyCode::Right {
                    return self.check_complete_default();
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

                self.validate_on_key(n);

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
                    Some((default, _)) if self.input.value().is_empty() => default,
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
                    prompt: widgets::Prompt::new(message),
                    input: widgets::StringInput::with_filter_map(Self::filter_map),
                    is_valid: true,
                    number: self,
                    answers,
                }
            }

            pub(crate) fn ask<B: Backend, E: EventIterator>(
                mut self,
                message: String,
                on_esc: ui::OnEsc,
                answers: &Answers,
                b: &mut B,
                events: &mut E,
            ) -> ui::Result<Option<Answer>> {
                let transform = self.transform.take();

                let ans = ui::Input::new(self.into_prompt(&message, answers), b)
                    .on_esc(on_esc)
                    .run(events)?;

                crate::write_final!(transform, message, ans, answers, b, |ans| Self::write(
                    ans, b
                )?)
            }
        }
    };
}

impl_ask!(Int, IntPrompt);
impl_ask!(Float, FloatPrompt);

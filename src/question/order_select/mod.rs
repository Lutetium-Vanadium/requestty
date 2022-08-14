mod builder;

use std::io;

use ui::{
    backend::Backend,
    events::EventIterator,
    style::Color,
    widgets::{self, List, Text},
    Prompt, Widget,
};

use crate::{Answer, Answers, ListItem};

use super::{
    handler::{Filter, Transform, Validate},
    Choice,
};

pub use builder::OrderSelectBuilder;

// =============================================================================
//
// =============================================================================

#[derive(Debug, Default)]
pub(super) struct OrderSelect<'a> {
    choices: super::ChoiceList<Text<String>>,
    max_number_len: usize,
    order: Vec<usize>,
    moving: bool,

    transform: Transform<'a, [ListItem]>,
    validate: Validate<'a, [usize]>,
    filter: Filter<'a, Vec<usize>>,
}

impl widgets::List for OrderSelect<'_> {
    fn render_item<B: ui::backend::Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> std::io::Result<()> {
        let symbol_set = ui::symbols::current();

        if hovered {
            if self.moving {
                b.set_bg(Color::Cyan)?;
                b.set_fg(Color::Black)?;
            } else {
                b.set_fg(Color::Cyan)?;
            }

            write!(b, "{} ", symbol_set.pointer)?;
        } else {
            b.write_all(b"  ")?;
        }

        layout.offset_x += 4;

        let mut index_str = (index + 1).to_string();
        index_str.extend(
            ' '.to_string()
                .repeat(self.max_number_len - index_str.len())
                .chars()
        );

        write!(b, "{} ", index_str)?;

        layout.offset_x += 2;

        self.choices[self.order[index]].render(&mut layout, b)?;

        b.set_fg(Color::Reset)?;
        b.set_bg(Color::Reset)
    }

    fn is_selectable(&self, _index: usize) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        self.choices.page_size()
    }

    fn should_loop(&self) -> bool {
        self.choices.should_loop()
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        layout.offset_x += 4;
        self.choices[index].height(&mut layout)
    }

    fn len(&self) -> usize {
        self.choices.len()
    }
}

impl<'c> OrderSelect<'c> {
    fn into_order_select_prompt<'a>(
        self,
        message: &'a str,
        answers: &'a Answers,
    ) -> OrderSelectPrompt<'a, 'c> {
        OrderSelectPrompt {
            prompt: widgets::Prompt::new(message).with_hint(
                "Press <space> to take and place an option, and <up> and <down> to move",
            ),
            select: widgets::Select::new(self),
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

        let ans = ui::Input::new(self.into_order_select_prompt(&message, answers), b)
            .hide_cursor()
            .on_esc(on_esc)
            .run(events)?;

        crate::write_final!(transform, message, ans [ref], answers, b, |ans| {
            b.set_fg(Color::Cyan)?;
            print_comma_separated(
                ans.iter().map(|item| {
                    item.text
                        .lines()
                        .next()
                        .expect("There must be at least one line in a `str`")
                }),
                b,
            )?;
            b.set_fg(Color::Reset)?;
        })
    }
}

fn print_comma_separated<'a, B: Backend>(
    iter: impl Iterator<Item = &'a str>,
    b: &mut B,
) -> io::Result<()> {
    let mut iter = iter.peekable();

    while let Some(item) = iter.next() {
        b.write_all(item.as_bytes())?;
        if iter.peek().is_some() {
            b.write_all(b", ")?;
        }
    }

    Ok(())
}

// =============================================================================
//
// =============================================================================

struct OrderSelectPrompt<'a, 'c> {
    prompt: widgets::Prompt<&'a str>,
    select: widgets::Select<OrderSelect<'c>>,
    answers: &'a Answers,
}

impl Prompt for OrderSelectPrompt<'_, '_> {
    type ValidateErr = widgets::Text<String>;
    type Output = Vec<ListItem>;

    fn finish(self) -> Self::Output {
        let OrderSelect {
            choices,
            mut order,
            filter,
            ..
        } = self.select.into_inner();

        if let Filter::Sync(filter) = filter {
            order = filter(order, self.answers);
        }

        order
            .into_iter()
            .filter_map(|i| match &choices.choices[i] {
                Choice::Choice(text) => Some(ListItem {
                    index: i,
                    text: text.text.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    fn validate(&mut self) -> Result<ui::Validation, Self::ValidateErr> {
        if let Validate::Sync(ref mut validate) = self.select.list.validate {
            validate(&self.select.list.order, self.answers)?;
        }
        Ok(ui::Validation::Finish)
    }
}

macro_rules! key_swap {
    ($self: expr, $key: expr, $go_back_if: expr, $go_back_val: expr, $op: tt) => {
        {
            if $self.select.list.moving {
                let mut should_swap = true;
                let p1 = $self.select.get_at();
                let p2 = if p1 == $go_back_if {
                    if $self.select.list.should_loop() {
                        $go_back_val
                    } else {
                        should_swap = false;
                        p1
                    }
                } else {
                    p1 $op 1
                };

                if should_swap {
                    $self.select.list.order.swap(p1, p2);
                }
            }
            $self.select.handle_key($key);
        }
    };
}

impl Widget for OrderSelectPrompt<'_, '_> {
    fn render<B: Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        backend: &mut B,
    ) -> io::Result<()> {
        self.prompt.render(layout, backend)?;
        self.select.render(layout, backend)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.prompt.height(layout) + self.select.height(layout) - 1
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.select.cursor_pos(layout)
    }

    fn handle_key(&mut self, key: ui::events::KeyEvent) -> bool {
        match key.code {
            ui::events::KeyCode::Up => {
                key_swap!(self, key, 0, self.select.list.choices.len() - 1, -)
            }
            ui::events::KeyCode::Down => {
                key_swap!(self, key, self.select.list.choices.len() - 1, 0, +)
            }

            ui::events::KeyCode::Char(' ') => self.select.list.moving = !self.select.list.moving,
            _ => return self.select.handle_key(key),
        }

        true
    }
}

use std::io::Write;

use crossterm::{
    cursor, event, queue,
    style::{Colorize, PrintStyledContent},
    terminal,
};

use crate::widget::Widget;

/// A trait to represent a renderable list
pub trait List {
    /// Render a single element at some index in **only** one line
    fn render_item<W: Write>(
        &mut self,
        index: usize,
        hovered: bool,
        max_width: usize,
        w: &mut W,
    ) -> crossterm::Result<()>;

    /// Whether the element at a particular index is selectable. Those that are not selectable are
    /// skipped over when the navigation keys are used
    fn is_selectable(&self, index: usize) -> bool;

    /// The maximum height that can be taken by the list. If there are more elements than the page
    /// size, the list will be scrollable
    fn page_size(&self) -> usize;
    /// Whether to wrap around when user gets to the last element
    fn should_loop(&self) -> bool;

    /// The length of the list
    fn len(&self) -> usize;
    /// Returns true if the list has no elements
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A widget to select a single item from a list. It can also be used to generally keep track of
/// movements within a list.
pub struct ListPicker<L> {
    first_selectable: usize,
    last_selectable: usize,
    at: usize,
    page_start: usize,
    /// The underlying list
    pub list: L,
}

impl<L: List> ListPicker<L> {
    /// Creates a new [`ListPicker`]
    pub fn new(list: L) -> Self {
        let first_selectable = (0..list.len())
            .position(|i| list.is_selectable(i))
            .expect("there must be at least one selectable item");

        let last_selectable = (0..list.len())
            .rposition(|i| list.is_selectable(i))
            .unwrap();

        assert!(list.page_size() >= 5, "page size can be a minimum of 5");

        Self {
            first_selectable,
            last_selectable,
            at: first_selectable,
            page_start: 0,
            list,
        }
    }

    /// The index of the element that is currently being hovered
    pub fn get_at(&self) -> usize {
        self.at
    }

    /// Set the index of the element that is currently being hovered
    pub fn set_at(&mut self, at: usize) {
        let dir = if self.at > self.list.len() || self.at < at {
            Direction::Down
        } else {
            Direction::Up
        };

        self.at = at;

        if self.is_paginating() {
            if at >= self.list.len() {
                self.page_start = 0;
            } else {
                self.adjust_page_start(dir);
            }
        }
    }

    /// Consumes the list picker returning the original list. If you need the selected item, use
    /// [`get_at`](ListPicker::get_at)
    pub fn finish(self) -> L {
        self.list
    }

    fn next_selectable(&self) -> usize {
        if self.at == self.last_selectable {
            return if self.list.should_loop() {
                self.first_selectable
            } else {
                self.at
            };
        }

        let mut at = self.at;
        loop {
            at = (at + 1).min(self.list.len()) % self.list.len();
            if self.list.is_selectable(at) {
                break;
            }
        }
        at
    }

    fn prev_selectable(&self) -> usize {
        if self.at == self.first_selectable {
            return if self.list.should_loop() {
                self.last_selectable
            } else {
                self.at
            };
        }

        let mut at = self.at;
        loop {
            at = (self.list.len() + at.min(self.list.len()) - 1) % self.list.len();
            if self.list.is_selectable(at) {
                break;
            }
        }
        at
    }

    fn is_paginating(&self) -> bool {
        self.list.len() > self.list.page_size()
    }

    fn adjust_page_start(&mut self, moved: Direction) {
        // Check whether at is within second and second last element of the page
        if self.at <= self.page_start || self.at >= self.page_start + self.list.page_size() - 2 {
            self.page_start = match moved {
                // At end of the list, but shouldn't loop, so the last element should be at the end
                // of the page
                Direction::Down if !self.list.should_loop() && self.at == self.list.len() - 1 => {
                    self.list.len() - self.list.page_size() + 1
                }
                // Make sure cursor is at second last element of the page
                Direction::Down => {
                    (self.list.len() + self.at + 3 - self.list.page_size()) % self.list.len()
                }
                // At start of the list, but shouldn't loop, so the first element should be at the
                // start of the page
                Direction::Up if !self.list.should_loop() && self.at == 0 => 0,
                // Make sure cursor is at second element of the page
                Direction::Up => (self.at + self.list.len() - 1) % self.list.len(),
            }
        }
    }

    /// Renders the lines in a given iterator
    fn render_in<W: Write>(
        &mut self,
        iter: impl Iterator<Item = usize>,
        w: &mut W,
    ) -> crossterm::Result<()> {
        let max_width = terminal::size()?.0 as usize;

        for i in iter {
            self.list.render_item(i, i == self.at, max_width, w)?;
            queue!(w, cursor::MoveToNextLine(1))?;
        }

        Ok(())
    }
}

impl<L: List> Widget for ListPicker<L> {
    /// It handles the following keys:
    /// - Up and 'k' to move up to the previous selectable element
    /// - Down and 'j' to move up to the next selectable element
    /// - Home, PageUp and 'g' to go to the first selectable element
    /// - End, PageDown and 'G' to go to the last selectable element
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        let moved = match key.code {
            event::KeyCode::Up | event::KeyCode::Char('k') => {
                self.at = self.prev_selectable();
                Direction::Up
            }
            event::KeyCode::Down | event::KeyCode::Char('j') => {
                self.at = self.next_selectable();
                Direction::Down
            }

            event::KeyCode::PageUp
                if !self.is_paginating() // No pagination, PageUp is same as Home
                    // No looping, and first item is shown in this page
                    || (!self.list.should_loop() && self.at + 2 < self.list.page_size()) =>
            {
                self.at = self.first_selectable;
                Direction::Up
            }
            event::KeyCode::PageUp => {
                self.at = (self.list.len() + self.at + 2 - self.list.page_size()) % self.list.len();
                self.at = self.next_selectable();
                Direction::Up
            }

            event::KeyCode::PageDown
                if !self.is_paginating() // No pagination, PageDown same as End
                    || (!self.list.should_loop() // No looping and last item is shown in this page
                        && self.at + self.list.page_size() - 2 >= self.list.len()) =>
            {
                self.at = self.last_selectable;
                Direction::Down
            }
            event::KeyCode::PageDown => {
                self.at = (self.at + self.list.page_size() - 2) % self.list.len();
                self.at = self.prev_selectable();
                Direction::Down
            }

            event::KeyCode::Home | event::KeyCode::Char('g') if self.at != 0 => {
                self.at = self.first_selectable;
                Direction::Up
            }
            event::KeyCode::End | event::KeyCode::Char('G') if self.at != self.list.len() - 1 => {
                self.at = self.last_selectable;
                Direction::Down
            }

            _ => return false,
        };

        if self.is_paginating() {
            self.adjust_page_start(moved)
        }

        true
    }

    fn render<W: Write>(&mut self, _: usize, w: &mut W) -> crossterm::Result<()> {
        queue!(w, cursor::MoveToNextLine(1))?;

        if self.is_paginating() {
            let end_iter =
                self.page_start..(self.page_start + self.list.page_size() - 1).min(self.list.len());

            if self.list.should_loop() {
                // Since we should loop, we need to chain the start of the list as well
                let end_iter_len = end_iter.size_hint().0;
                self.render_in(
                    end_iter.chain(0..(self.list.page_size() - end_iter_len - 1)),
                    w,
                )?;
            } else {
                self.render_in(end_iter, w)?;
            }
        } else {
            self.render_in(0..self.list.len(), w)?;
        };

        if self.is_paginating() {
            queue!(
                w,
                PrintStyledContent("(Move up and down to reveal more choices)".dark_grey()),
                cursor::MoveToNextLine(1)
            )?;
        }

        Ok(())
    }

    fn cursor_pos(&self, _: u16) -> (u16, u16) {
        (0, 1 + self.at as u16)
    }

    fn height(&self) -> usize {
        self.list.len().min(self.list.page_size())
    }
}

enum Direction {
    Up,
    Down,
}

use crate::{
    backend::{Backend, MoveDirection, Stylize},
    error,
    events::{KeyEvent, Movement},
    Layout,
};

/// A trait to represent a renderable list
pub trait List {
    // FIXME: allow multi-line
    /// Render a single element at some index in **only** one line
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        layout: Layout,
        backend: &mut B,
    ) -> error::Result<()>;

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
            Movement::Down
        } else {
            Movement::Up
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

    fn adjust_page_start(&mut self, moved: Movement) {
        // Check whether at is within second and second last element of the page
        if self.at <= self.page_start
            || self.at >= self.page_start + self.list.page_size() - 2
        {
            self.page_start = match moved {
                // At end of the list, but shouldn't loop, so the last element should be at the end
                // of the page
                Movement::Down
                    if !self.list.should_loop()
                        && self.at == self.list.len() - 1 =>
                {
                    self.list.len() - self.list.page_size() + 1
                }
                // Make sure cursor is at second last element of the page
                Movement::Down => {
                    (self.list.len() + self.at + 3 - self.list.page_size())
                        % self.list.len()
                }
                // At start of the list, but shouldn't loop, so the first element should be at the
                // start of the page
                Movement::Up if !self.list.should_loop() && self.at == 0 => 0,
                // Make sure cursor is at second element of the page
                Movement::Up => (self.at + self.list.len() - 1) % self.list.len(),
                _ => unreachable!(),
            }
        }
    }

    /// Renders the lines in a given iterator
    fn render_in<B: Backend>(
        &mut self,
        iter: impl Iterator<Item = usize>,
        mut layout: Layout,
        b: &mut B,
    ) -> error::Result<()> {
        for i in iter {
            layout.offset_y += 1;
            self.list.render_item(i, i == self.at, layout, b)?;
            b.move_cursor(MoveDirection::NextLine(1))?;
        }

        Ok(())
    }
}

impl<L: List> super::Widget for ListPicker<L> {
    /// It handles the `Up`, `Down`, `Home`, `End`, `PageUp` and `PageDown` [`Movements`].
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let movement = match Movement::try_from_key(key) {
            Some(movement) => movement,
            None => return false,
        };

        let moved = match movement {
            Movement::Up => {
                self.at = self.prev_selectable();
                Movement::Up
            }
            Movement::Down => {
                self.at = self.next_selectable();
                Movement::Down
            }

            Movement::PageUp
                if !self.is_paginating() // No pagination, PageUp is same as Home
                    // No looping, and first item is shown in this page
                    || (!self.list.should_loop() && self.at + 2 < self.list.page_size()) =>
            {
                self.at = self.first_selectable;
                Movement::Up
            }
            Movement::PageUp => {
                self.at = (self.list.len() + self.at + 2 - self.list.page_size())
                    % self.list.len();
                self.at = self.next_selectable();
                Movement::Up
            }

            Movement::PageDown
                if !self.is_paginating() // No pagination, PageDown same as End
                    || (!self.list.should_loop() // No looping and last item is shown in this page
                        && self.at + self.list.page_size() - 2 >= self.list.len()) =>
            {
                self.at = self.last_selectable;
                Movement::Down
            }
            Movement::PageDown => {
                self.at = (self.at + self.list.page_size() - 2) % self.list.len();
                self.at = self.prev_selectable();
                Movement::Down
            }

            Movement::Home if self.at != self.first_selectable => {
                self.at = self.first_selectable;
                Movement::Up
            }
            Movement::End if self.at != self.last_selectable => {
                self.at = self.last_selectable;
                Movement::Down
            }

            _ => return false,
        };

        if self.is_paginating() {
            self.adjust_page_start(moved)
        }

        true
    }

    fn render<B: Backend>(
        &mut self,
        mut layout: Layout,
        b: &mut B,
    ) -> error::Result<()> {
        // TODO: allow multi-line options
        b.move_cursor(MoveDirection::NextLine(1))?;
        layout.line_offset = 0;
        layout.offset_y += 1;

        if self.is_paginating() {
            let end_iter = self.page_start
                ..(self.page_start + self.list.page_size() - 1).min(self.list.len());

            if self.list.should_loop() {
                // Since we should loop, we need to chain the start of the list as well
                let end_iter_len = end_iter.size_hint().0;
                self.render_in(
                    end_iter.chain(0..(self.list.page_size() - end_iter_len - 1)),
                    layout,
                    b,
                )?;
            } else {
                self.render_in(end_iter, layout, b)?;
            }
        } else {
            self.render_in(0..self.list.len(), layout, b)?;
        };

        if self.is_paginating() {
            b.write_styled("(Move up and down to reveal more choices)".dark_grey())?;
            b.move_cursor(MoveDirection::NextLine(1))?;
        }

        Ok(())
    }

    fn cursor_pos(&mut self, _: Layout) -> (u16, u16) {
        (0, 1 + self.at as u16)
    }

    fn height(&mut self, _: Layout) -> u16 {
        self.list.len().min(self.list.page_size()) as u16
    }
}

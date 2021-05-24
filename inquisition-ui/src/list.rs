use crate::{
    backend::{Backend, MoveDirection, Stylize},
    error,
    events::{KeyEvent, Movement},
    Layout,
};

/// A trait to represent a renderable list
pub trait List {
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

    /// The height of the element that an index will take to render
    fn height_at(&mut self, index: usize, layout: Layout) -> u16;

    /// The length of the list
    fn len(&self) -> usize;
    /// Returns true if the list has no elements
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

struct Heights {
    heights: Vec<u16>,
    prev_layout: Layout,
}

/// A widget to select a single item from a list. It can also be used to generally keep track of
/// movements within a list.
pub struct ListPicker<L> {
    first_selectable: usize,
    last_selectable: usize,
    at: usize,
    page_start: usize,
    page_end: usize,
    heights: Option<Heights>,
    height: u16,
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
            height: u16::MAX,
            heights: None,
            at: first_selectable,
            page_start: 0,
            page_end: usize::MAX,
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
                self.init_page_end();
            } else {
                self.adjust_page(dir);
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

    fn update_heights(&mut self, mut layout: Layout) {
        let heights = match self.heights {
            Some(ref mut heights) if heights.prev_layout != layout => {
                heights.heights.clear();
                heights.prev_layout = layout;
                &mut heights.heights
            }
            None => {
                self.heights = Some(Heights {
                    heights: Vec::with_capacity(self.list.len()),
                    prev_layout: layout,
                });

                &mut self.heights.as_mut().unwrap().heights
            }
            _ => return,
        };

        layout.line_offset = 0;

        self.height = 0;
        for i in 0..self.list.len() {
            let height = self.list.height_at(i, layout);
            self.height += height;
            heights.push(height);
        }
    }

    fn page_size(&self) -> u16 {
        self.list.page_size() as u16
    }

    fn is_paginating(&self) -> bool {
        self.height > self.page_size()
    }

    fn at_outside_page(&self) -> bool {
        if self.page_start < self.page_end {
            self.at <= self.page_start || self.at >= self.page_end
        } else {
            self.at <= self.page_start && self.at >= self.page_end
        }
    }

    /// Gets the index at a given delta taking into case wrapping if enabled -- delta
    /// must be within ±len
    fn try_get_index(&self, delta: isize) -> Option<usize> {
        if delta.is_positive() {
            let res = self.at + delta as usize;

            if res < self.list.len() {
                Some(res)
            } else if self.list.should_loop() {
                Some(res - self.list.len())
            } else {
                None
            }
        } else {
            let delta = -delta as usize;
            if self.list.should_loop() {
                Some((self.at + self.list.len() - delta) % self.list.len())
            } else {
                self.at.checked_sub(delta)
            }
        }
    }

    fn adjust_page(&mut self, moved: Movement) {
        // Check whether at is within second and second last element of the page
        let direction = if self.at_outside_page() {
            match moved {
                Movement::Down => -1,
                Movement::Up => 1,
                _ => unreachable!(),
            }
        } else {
            return;
        };

        let heights = &self.heights.as_ref().unwrap().heights[..];

        // -1 since the message at the end takes one line
        let max_height = self.page_size() - 1;

        // This first gets an element from the direction we have moved from, then one
        // from the opposite, and the rest again from the direction we have move from
        //
        // for example,
        // take that we have moved downwards (like from 2 to 3).
        // .-----.
        // |  0  | <-- iter[3]
        // .-----.
        // |  1  | <-- iter[2]
        // .-----.
        // |  2  | <-- iter[0] | We want this over 4 since we have come from that
        // .-----.               direction and it provides continuity
        // |  3  | <-- self.at
        // .-----.
        // |  4  | <-- iter[1] | We pick 4 over the earlier ones since it provides a
        // '-----'               padding of one element at the end
        //
        // note: the above example avoids things like looping, which is handled by
        // try_get_index
        let iter = self
            .try_get_index(direction)
            .map(|i| (i, false))
            .into_iter()
            .chain(
                self.try_get_index(-direction)
                    .map(|i| (i, true)) // boolean value to show this is special
                    .into_iter(),
            )
            .chain(
                (2..(max_height as isize))
                    .map(|i| self.try_get_index(direction * i).map(|i| (i, false)))
                    .flatten(),
            );

        // these numbers have opposite meaning based on the direction
        let mut bounds = (self.at, self.at);

        let mut height = heights[self.at];

        for (height_index, opposite_dir) in iter {
            if height + heights[height_index] > max_height {
                // There are no more elements that can be shown
                break;
            }

            // If you see the creation of iter, this special cases the second element in
            // the iterator as it is the _only_ one in the opposite direction
            //
            // It cannot simply be checked as being the second element, as try_get_index
            // may return None when looping is disabled
            if opposite_dir {
                bounds.1 = height_index;
            } else {
                bounds.0 = height_index;
            }

            height += heights[height_index];
        }

        // There is more space, try adding stuff from the opposite direction
        if height < max_height {
            let iter = (2..(max_height as isize))
                .map(|i| self.try_get_index(-direction * i))
                .flatten();

            for height_index in iter {
                if height + heights[height_index] > max_height {
                    // There are no more elements that can be shown
                    break;
                }

                bounds.1 = height_index;
                height += heights[height_index];
            }
        }

        if let Movement::Down = moved {
            // When moving down, the special case is the element after `self.at`, so it
            // is the page_end
            self.page_start = bounds.0;
            self.page_end = bounds.1;
        } else {
            // When moving up, the special case is the element before `self.at`, so it
            // is the page_start
            self.page_start = bounds.1;
            self.page_end = bounds.0;
        }
    }

    fn init_page_end(&mut self) {
        if self.is_paginating() {
            let heights = &self.heights.as_ref().unwrap().heights[..];
            let mut page_end = 0;
            let mut height = heights[0];
            // -1 since the message at the end takes one line
            let max_height = self.page_size() - 1;

            #[allow(clippy::needless_range_loop)]
            for i in 1..heights.len() {
                if height + heights[i] > max_height {
                    break;
                }
                height += heights[i];
                page_end = i;
            }

            self.page_end = page_end;
        } else {
            self.page_end = self.list.len() - 1;
        }
    }

    /// Renders the lines in a given iterator
    fn render_in<B: Backend>(
        &mut self,
        iter: impl Iterator<Item = usize> + std::fmt::Debug,
        mut layout: Layout,
        b: &mut B,
    ) -> error::Result<()> {
        let heights = &self.heights.as_ref().unwrap().heights[..];

        for i in iter {
            self.list.render_item(i, i == self.at, layout, b)?;
            b.move_cursor(MoveDirection::NextLine(1))?;
            layout.offset_y += heights[i];
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
            self.adjust_page(moved)
        }

        true
    }

    fn render<B: Backend>(
        &mut self,
        mut layout: Layout,
        b: &mut B,
    ) -> error::Result<()> {
        self.update_heights(layout);

        // this is the first render, so we need to set page_end
        if self.page_end == usize::MAX {
            self.init_page_end();
        }

        b.move_cursor(MoveDirection::NextLine(1))?;
        layout.line_offset = 0;
        layout.offset_y += 1;

        if self.page_end < self.page_start {
            self.render_in(
                (self.page_start..self.list.len()).chain(0..=self.page_end),
                layout,
                b,
            )?;
        } else {
            self.render_in(self.page_start..=self.page_end, layout, b)?;
        }

        if self.is_paginating() {
            // This is the message at the end that other places refer to
            b.write_styled("(Move up and down to reveal more choices)".dark_grey())?;
            b.move_cursor(MoveDirection::NextLine(1))?;
        }

        Ok(())
    }

    fn cursor_pos(&mut self, _: Layout) -> (u16, u16) {
        unimplemented!("list does not support cursor pos")
    }

    fn height(&mut self, layout: Layout) -> u16 {
        self.update_heights(layout);

        // Try to show everything
        self.height
            // otherwise show whatever is possible
            .min(self.page_size())
            // but do not show less than a single element
            // +1 since the message at the end takes one line
            .max(
                self.heights
                    .as_ref()
                    .unwrap()
                    .heights
                    .get(self.at)
                    .unwrap_or(&0)
                    + 1,
            )
    }
}

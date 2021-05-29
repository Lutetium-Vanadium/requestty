use std::ops::{Index, IndexMut};

use crate::{
    backend::{Backend, MoveDirection, Stylize},
    error,
    events::{KeyEvent, Movement},
    Layout, RenderRegion,
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
    page_start_height: u16,
    page_end_height: u16,
    height: u16,
    heights: Option<Heights>,
    /// The underlying list
    pub list: L,
}

impl<L: Index<usize>> ListPicker<L> {
    pub fn selected(&self) -> &L::Output {
        &self.list[self.at]
    }
}

impl<L: IndexMut<usize>> ListPicker<L> {
    pub fn selected_mut(&mut self) -> &mut L::Output {
        &mut self.list[self.at]
    }
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
            page_start_height: u16::MAX,
            page_end_height: u16::MAX,
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
                self.init_page();
            } else {
                self.try_adjust_page(dir);
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
        let direction = match moved {
            Movement::Down => -1,
            Movement::Up => 1,
            _ => unreachable!(),
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

        // these variables have opposite meaning based on the direction, but they store
        // the (index, height) of either the page_start or the page_end
        let mut bound_a = (self.at, heights[self.at]);
        let mut bound_b = (self.at, heights[self.at]);

        let mut height = heights[self.at];

        for (height_index, opposite_dir) in iter {
            if height >= max_height {
                // There are no more elements that can be shown
                break;
            }

            let elem_height = if opposite_dir {
                // To provide better continuity, the element in the opposite direction
                // will have only one line shown. This prevents the cursor from jumping
                // about when the element in the opposite direction has different height
                // from the one rendered previously
                1
            } else {
                (height + heights[height_index]).min(max_height) - height
            };

            // If you see the creation of iter, this special cases the second element in
            // the iterator as it is the _only_ one in the opposite direction
            //
            // It cannot simply be checked as being the second element, as try_get_index
            // may return None when looping is disabled
            if opposite_dir {
                bound_b.0 = height_index;
                bound_b.1 = elem_height;
            } else {
                bound_a.0 = height_index;
                bound_a.1 = elem_height;
            }

            height += elem_height;
        }

        if let Movement::Down = moved {
            // When moving down, the special case is the element after `self.at`, so it
            // is the page_end
            self.page_start = bound_a.0;
            self.page_start_height = bound_a.1;
            self.page_end = bound_b.0;
            self.page_end_height = bound_b.1;
        } else {
            // When moving up, the special case is the element before `self.at`, so it
            // is the page_start
            self.page_start = bound_b.0;
            self.page_start_height = bound_b.1;
            self.page_end = bound_a.0;
            self.page_end_height = bound_a.1;
        }
    }

    fn try_adjust_page(&mut self, moved: Movement) {
        // Check whether at is within second and second last element of the page
        if self.at_outside_page() {
            self.adjust_page(moved)
        }
    }

    fn init_page(&mut self) {
        let heights = &self.heights.as_ref().unwrap().heights[..];

        self.page_start = 0;
        self.page_start_height = heights[self.page_start];

        if self.is_paginating() {
            let mut height = heights[0];
            // -1 since the message at the end takes one line
            let max_height = self.page_size() - 1;

            #[allow(clippy::needless_range_loop)]
            for i in 1..heights.len() {
                if height >= max_height {
                    break;
                }
                self.page_end = i;
                self.page_end_height =
                    (height + heights[i]).min(max_height) - height;

                height += heights[i];
            }
        } else {
            self.page_end = self.list.len() - 1;
            self.page_end_height = heights[self.page_end];
        }
    }

    /// Renders the lines in a given iterator
    fn render_in<B: Backend>(
        &mut self,
        iter: impl Iterator<Item = usize>,
        mut layout: Layout,
        b: &mut B,
    ) -> error::Result<()> {
        let heights = &self.heights.as_ref().unwrap().heights[..];

        for i in iter {
            if i == self.page_start {
                layout.max_height = self.page_start_height;
                layout.render_region = RenderRegion::Bottom;
            } else if i == self.page_end {
                layout.max_height = self.page_end_height;
                layout.render_region = RenderRegion::Top;
            } else {
                layout.max_height = heights[i];
            }

            self.list.render_item(i, i == self.at, layout, b)?;
            b.move_cursor(MoveDirection::NextLine(1))?;

            layout.offset_y += layout.max_height;
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
                    // No looping and first item is shown in this page
                    || (!self.list.should_loop() && self.page_start == 0) =>
            {
                self.at = self.first_selectable;
                Movement::Up
            }
            Movement::PageUp => {
                // We want the current self.at to be visible after the PageUp movement,
                // and if possible we want to it to be the bottom most element visible

                // We decrease self.at by 1, since adjust_page will put self.at as the
                // second last element, so if (self.at - 1) is the second last element,
                // self.at is the last element visible
                self.at = self.try_get_index(-1).unwrap_or(self.at);
                self.adjust_page(Movement::Down);

                if self.page_start == 0 && !self.list.should_loop() {
                    // We've reached the end, it is possible that because of the bounds
                    // we gave earlier, self.page_end may not be right so we have to
                    // recompute it
                    self.at = self.first_selectable;
                    self.init_page();
                } else {
                    // Now that the page is determined, we want to set self.at to be some
                    // _selectable_ element which is not the top most element visible,
                    // so we undershoot by 1
                    self.at = self.page_start;
                    // ...and then go forward to at least one element
                    //
                    // note: self.at cannot directly be set to self.page_start + 1, since it
                    // also has to be a selectable element
                    self.at = self.next_selectable();
                }

                Movement::Up
            }

            Movement::PageDown
                if !self.is_paginating() // No pagination, PageDown same as End
                    || (!self.list.should_loop() // No looping and last item is shown in this page
                        && self.page_end + 1 == self.list.len()) =>
            {
                self.at = self.last_selectable;
                Movement::Down
            }
            Movement::PageDown => {
                // We want the current self.at to be visible after the PageDown movement,
                // and if possible we want to it to be the top most element visible

                // We increase self.at by 1, since adjust_page will put self.at as the
                // second element, so if (self.at + 1) is the second last element,
                // self.at is the last element visible
                self.at = self.try_get_index(1).unwrap_or(self.at);
                self.adjust_page(Movement::Up);

                if self.page_end + 1 == self.list.len() {
                    // We've reached the end, it is possible that because of the bounds
                    // we gave earlier, self.page_start may not be right so we have to
                    // recompute it
                    self.at = self.page_end;
                    self.adjust_page(Movement::Down);
                    self.at = self.last_selectable;
                } else {
                    // Now that the page is determined, we want to set self.at to be some
                    // _selectable_ element which is not the bottom most element visible,
                    // so we overshoot by 1
                    self.at = self.page_end;
                    // ...and then go back to at least one element
                    //
                    // note: self.at cannot directly be set to self.page_end - 1, since it
                    // also has to be a selectable element
                    self.at = self.prev_selectable();
                }

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
            self.try_adjust_page(moved)
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
            self.init_page();
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
            b.write_styled(
                &"(Move up and down to reveal more choices)".dark_grey(),
            )?;
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
        1 + self // Add one since we go to the next line
            .height
            // otherwise show whatever is possible
            .min(self.page_size())
            // but do not show less than a single element
            .max(
                self.heights
                    .as_ref()
                    .unwrap()
                    .heights
                    .get(self.at)
                    .unwrap_or(&0)
                    // +1 if paginating since the message at the end takes one line
                    + self.is_paginating() as u16,
            )
    }
}

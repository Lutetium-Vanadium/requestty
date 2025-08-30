use std::{
    io,
    ops::{Index, IndexMut},
};

use crate::{
    backend::Backend,
    events::{KeyEvent, Movement},
    layout::{Layout, RenderRegion},
    style::Stylize,
};

#[cfg(test)]
mod tests;

/// A trait to represent a renderable list.
///
/// See [`Select`]
pub trait List {
    /// Render a single element at some index.
    ///
    /// When rendering the element, only _at most_ [`layout.max_height`] lines can be used. If more
    /// lines are used, the list may not be rendered properly. The place the terminal cursor ends at
    /// does not matter.
    ///
    /// [`layout.max_height`] may be less than the height given by [`height_at`].
    /// [`layout.render_region`] can be used to determine which part of the element you want to
    /// render.
    ///
    /// [`height_at`]: List::height_at
    /// [`layout.max_height`]: Layout::max_height
    /// [`layout.render_region`]: Layout.render_region
    fn render_item<B: Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        layout: Layout,
        backend: &mut B,
    ) -> io::Result<()>;

    /// Whether the element at a particular index is selectable. Those that are not selectable are
    /// skipped during navigation.
    fn is_selectable(&self, index: usize) -> bool;

    /// The maximum height that can be taken by the list.
    ///
    /// If the total height exceeds the page size, the list will be scrollable.
    fn page_size(&self) -> usize;

    /// Whether to wrap around when user gets to the last element.
    ///
    /// This only applies when the list is scrollable, i.e. page size > total height.
    fn should_loop(&self) -> bool;

    /// The height of the element at an index will take to render
    fn height_at(&mut self, index: usize, layout: Layout) -> u16;

    /// The length of the list
    fn len(&self) -> usize;

    /// Returns true if the list has no elements
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
struct Heights {
    heights: Vec<u16>,
    prev_layout: Layout,
}

/// A widget to select a single item from a list.
///
/// The list must implement the [`List`] trait.
#[derive(Debug, Clone)]
pub struct Select<L> {
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

impl<L: List> Select<L> {
    /// Creates a new [`Select`].
    ///
    /// # Panics
    ///
    /// Panics if there are no selectable items, or if `list.page_size()` is less than 5.
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

    /// The index of the element that is currently being hovered.
    pub fn get_at(&self) -> usize {
        self.at
    }

    /// Set the index of the element that is currently being hovered.
    ///
    /// `at` can be any number (even beyond `list.len()`), but the caller is responsible for making
    /// sure that it is a selectable element.
    pub fn set_at(&mut self, at: usize) {
        let dir = if self.at >= self.list.len() || self.at < at {
            Movement::Down
        } else {
            Movement::Up
        };

        self.at = at;

        if self.is_paginating() {
            if at >= self.list.len() {
                self.init_page();
            } else if self.heights.is_some() {
                self.maybe_adjust_page(dir);
            }
        }
    }

    /// Consumes the [`Select`] returning the original list.
    pub fn into_inner(self) -> L {
        self.list
    }

    fn next_selectable(&self) -> usize {
        if self.at >= self.last_selectable {
            return if self.list.should_loop() {
                self.first_selectable
            } else {
                self.last_selectable
            };
        }

        // at not guaranteed to be in the valid range of 0..list.len(), so the min is required
        let mut at = self.at.min(self.list.len());
        loop {
            at = (at + 1) % self.list.len();
            if self.list.is_selectable(at) {
                break;
            }
        }
        at
    }

    fn prev_selectable(&self) -> usize {
        if self.at <= self.first_selectable {
            return if self.list.should_loop() {
                self.last_selectable
            } else {
                self.first_selectable
            };
        }

        // at not guaranteed to be in the valid range of 0..list.len(), so the min is required
        let mut at = self.at.min(self.list.len());
        loop {
            at = (self.list.len() + at - 1) % self.list.len();
            if self.list.is_selectable(at) {
                break;
            }
        }
        at
    }

    fn maybe_update_heights(&mut self, mut layout: Layout) {
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

    /// Checks whether the page bounds need to be adjusted
    ///
    /// This returns true if at == page_start || at == page_end, and so even though it is visible,
    /// the page bounds should be adjusted
    fn at_outside_page(&self) -> bool {
        if self.page_start < self.page_end {
            // - a - - S - - - - - - E - a -
            //   ^------- outside -------^
            self.at <= self.page_start || self.at >= self.page_end
        } else {
            // - - - - E - - - a - - S - - -
            //       outside --^
            self.at <= self.page_start && self.at >= self.page_end
        }
    }

    /// Gets the index at a given delta taking into account looping if enabled -- delta must be
    /// within ±len
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

    /// Adjust the page considering the direction we moved to
    fn adjust_page(&mut self, moved_to: Movement) {
        // note direction here refers to the direction we moved _from_, while moved means the
        // direction we moved _to_, and so they have opposite meanings
        let direction = match moved_to {
            Movement::Down => -1,
            Movement::Up => 1,
            _ => unreachable!(),
        };

        let heights = &self
            .heights
            .as_ref()
            .expect("`adjust_page` called before `height` or `render`")
            .heights[..];

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
        // |  4  | <-- iter[1] | We pick 4 over ones before 2 since it provides a
        // '-----'               padding of one element at the end
        //
        // note: the above example avoids things like looping, which is handled by
        // try_get_index
        let iter = self
            .try_get_index(direction)
            .map(|i| (i, false))
            .into_iter()
            .chain(
                self.try_get_index(-direction).map(|i| (i, true)), // boolean value to show this is special
            )
            .chain(
                (2..(max_height as isize))
                    .filter_map(|i| self.try_get_index(direction * i).map(|i| (i, false))),
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

        if let Movement::Down = moved_to {
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

    /// Adjust the page if required considering the direction we moved to
    fn maybe_adjust_page(&mut self, moved_to: Movement) {
        // Check whether at is within second and second last element of the page
        if self.at_outside_page() {
            self.adjust_page(moved_to)
        }
    }

    fn init_page(&mut self) {
        let heights = &self
            .heights
            .as_ref()
            .expect("`init_page` called before `height` or `render`")
            .heights[..];

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
                self.page_end_height = (height + heights[i]).min(max_height) - height;

                height += heights[i];
            }
        } else {
            self.page_end = self.list.len() - 1;
            self.page_end_height = heights[self.page_end];
        }
    }

    /// Renders the lines in a given iterator
    fn render_in<I: Iterator<Item = usize>, B: Backend>(
        &mut self,
        iter: I,
        old_layout: &mut Layout,
        b: &mut B,
    ) -> io::Result<()> {
        let heights = &self
            .heights
            .as_ref()
            .expect("`render_in` called from someplace other than `render`")
            .heights[..];

        // Create a new local copy of the layout to operate on to avoid changes in max_height and
        // render_region to be reflected upstream
        let mut layout = *old_layout;

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
            layout.offset_y += layout.max_height;

            b.move_cursor_to(layout.offset_x, layout.offset_y)?;
        }

        old_layout.offset_y = layout.offset_y;
        layout.line_offset = 0;

        Ok(())
    }
}

impl<L: Index<usize>> Select<L> {
    /// Returns a reference to the currently hovered item.
    pub fn selected(&self) -> &L::Output {
        &self.list[self.at]
    }
}

impl<L: IndexMut<usize>> Select<L> {
    /// Returns a mutable reference to the currently hovered item.
    pub fn selected_mut(&mut self) -> &mut L::Output {
        &mut self.list[self.at]
    }
}

impl<L: List> super::Widget for Select<L> {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let movement = match Movement::try_from_key(key) {
            Some(movement) => movement,
            None => return false,
        };

        let moved = match movement {
            Movement::Up if self.list.should_loop() || self.at > self.first_selectable => {
                self.at = self.prev_selectable();
                Movement::Up
            }
            Movement::Down if self.list.should_loop() || self.at < self.last_selectable => {
                self.at = self.next_selectable();
                Movement::Down
            }

            Movement::PageUp
                if !self.is_paginating() // No pagination, PageUp is same as Home
                    // No looping and first item is shown in this page
                    || (!self.list.should_loop() && self.page_start == 0) =>
            {
                if self.at <= self.first_selectable {
                    return false;
                }
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
                    // ...and then go forward at least one element
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
                if self.at >= self.last_selectable {
                    return false;
                }
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

                // Now that the page is determined, we want to set self.at to be some
                // _selectable_ element which is not the bottom most element visible,
                // so we overshoot by 1...
                self.at = self.page_end;

                if self.page_end + 1 == self.list.len() && !self.list.should_loop() {
                    // ...but since we reached the end and there is no looping, self.page_start may
                    // not be right so we have to recompute it
                    self.adjust_page(Movement::Down);
                    self.at = self.last_selectable;
                } else {
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
            self.maybe_adjust_page(moved)
        }

        true
    }

    fn render<B: Backend>(&mut self, layout: &mut Layout, b: &mut B) -> io::Result<()> {
        self.maybe_update_heights(*layout);

        // this is the first render, so we need to set page_end
        if self.page_end == usize::MAX {
            self.init_page();
        }

        if layout.line_offset != 0 {
            layout.line_offset = 0;
            layout.offset_y += 1;
            b.move_cursor_to(layout.offset_x, layout.offset_y)?;
        }

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
            b.write_styled(&"(Move up and down to reveal more choices)".dark_grey())?;
            layout.offset_y += 1;

            b.move_cursor_to(layout.offset_x, layout.offset_y)?;
        }

        Ok(())
    }

    /// Returns the starting location of the layout. It should not be relied upon for a sensible
    /// cursor position.
    fn cursor_pos(&mut self, layout: Layout) -> (u16, u16) {
        layout.offset_cursor((layout.line_offset, 0))
    }

    fn height(&mut self, layout: &mut Layout) -> u16 {
        self.maybe_update_heights(*layout);

        let height = (layout.line_offset != 0) as u16 // Add one if we go to the next line
            // Try to show everything
            + self
                .height
                // otherwise show whatever is possible
                .min(self.page_size())
                // but do not show less than a single element
                .max(
                    self.heights
                    .as_ref()
                    .expect("`maybe_update_heights` should set `self.heights` if missing")
                    .heights
                    .get(self.at)
                    .unwrap_or(&0)
                    // +1 if paginating since the message at the end takes one line
                    + self.is_paginating() as u16,
                );

        layout.line_offset = 0;
        layout.offset_y += height;

        height
    }
}

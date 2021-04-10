use std::io::Write;

use crossterm::{cursor, event, queue, terminal};

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
    /// skipped over when the navigation keys are used.
    fn is_selectable(&self, index: usize) -> bool;

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

        Self {
            first_selectable,
            last_selectable,
            at: first_selectable,
            list,
        }
    }

    /// The index of the element that is currently being hovered
    pub fn get_at(&self) -> usize {
        self.at
    }

    /// Set the index of the element that is currently being hovered
    pub fn set_at(&mut self, at: usize) {
        self.at = at;
    }

    /// Consumes the list picker returning the original list. If you need the selected item, use
    /// [`get_at`](ListPicker::get_at)
    pub fn finish(self) -> L {
        self.list
    }

    fn next_selectable(&self) -> usize {
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
        let mut at = self.at;
        loop {
            at = (self.list.len() + at.min(self.list.len()) - 1) % self.list.len();
            if self.list.is_selectable(at) {
                break;
            }
        }
        at
    }
}

impl<L: List> Widget for ListPicker<L> {
    /// It handles the following keys:
    /// - Up and 'k' to move up to the previous selectable element
    /// - Down and 'j' to move up to the next selectable element
    /// - Home, PageUp and 'g' to go to the first selectable element
    /// - End, PageDown and 'G' to go to the last selectable element
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        match key.code {
            event::KeyCode::Up | event::KeyCode::Char('k') => {
                self.at = self.prev_selectable();
            }
            event::KeyCode::Down | event::KeyCode::Char('j') => {
                self.at = self.next_selectable();
            }

            event::KeyCode::Home | event::KeyCode::PageUp | event::KeyCode::Char('g')
                if self.at != 0 =>
            {
                self.at = self.first_selectable;
            }
            event::KeyCode::End | event::KeyCode::PageDown | event::KeyCode::Char('G')
                if self.at != self.list.len() - 1 =>
            {
                self.at = self.last_selectable;
            }

            _ => return false,
        }

        true
    }

    fn render<W: Write>(&mut self, _: usize, w: &mut W) -> crossterm::Result<()> {
        let max_width = terminal::size()?.0 as usize;
        queue!(w, cursor::MoveToNextLine(1))?;
        for i in 0..self.list.len() {
            self.list.render_item(i, i == self.at, max_width, w)?;
            queue!(w, cursor::MoveToNextLine(1))?;
        }

        Ok(())
    }

    fn cursor_pos(&self, _: u16) -> (u16, u16) {
        (0, 1 + self.at as u16)
    }

    fn height(&self) -> usize {
        self.list.len()
    }
}

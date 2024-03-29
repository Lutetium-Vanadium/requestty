use std::{
    io,
    ops::{Index, IndexMut},
};

use ui::{style::Color, widgets::List, Widget};

use crate::ExpandItem;

#[derive(Clone)]
pub(crate) struct SelectList<T> {
    pub(crate) choices: Vec<T>,
    page_size: usize,
    default: usize,
    // note: default is not an option usize because it adds an extra usize of space
    has_default: bool,
    should_loop: bool,
    is_selectable: fn(&T) -> bool,
}

impl<T: std::fmt::Debug> std::fmt::Debug for SelectList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectList")
            .field("choices", &self.choices)
            .field("page_size", &self.page_size)
            .field("default", &self.default)
            .field("has_default", &self.has_default)
            .field("should_loop", &self.should_loop)
            .finish()
    }
}

impl<T> SelectList<T> {
    pub(crate) fn new(f: fn(&T) -> bool) -> Self {
        Self {
            choices: Vec::new(),
            page_size: 15,
            default: 0,
            has_default: false,
            should_loop: true,
            is_selectable: f,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.choices.len()
    }

    /// Get a reference to the choice list's default.
    pub(crate) fn default(&self) -> Option<usize> {
        if self.has_default {
            Some(self.default)
        } else {
            None
        }
    }

    /// Get a reference to the choice list's page size.
    pub(crate) fn page_size(&self) -> usize {
        self.page_size
    }

    /// Get a reference to the choice list's should loop.
    pub(crate) fn should_loop(&self) -> bool {
        self.should_loop
    }

    /// Set the choice list's default.
    pub(crate) fn set_default(&mut self, default: usize) {
        self.default = default;
        self.has_default = true;
    }

    /// Set the choice list's page size.
    pub(crate) fn set_page_size(&mut self, page_size: usize) {
        self.page_size = page_size;
    }

    /// Set the choice list's should loop.
    pub(crate) fn set_should_loop(&mut self, should_loop: bool) {
        self.should_loop = should_loop;
    }
}

impl<T> Index<usize> for SelectList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.choices[index]
    }
}

impl<T> IndexMut<usize> for SelectList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.choices[index]
    }
}

impl<T: Widget> List for SelectList<T> {
    fn render_item<B: ui::backend::Backend>(
        &mut self,
        index: usize,
        hovered: bool,
        mut layout: ui::layout::Layout,
        b: &mut B,
    ) -> io::Result<()> {
        if hovered {
            b.set_fg(Color::Cyan)?;
            write!(b, "{} ", ui::symbols::current().pointer)?;
        } else {
            b.write_all(b"  ")?;

            if !self.is_selectable(index) {
                b.set_fg(Color::DarkGrey)?;
            }
        }

        layout.offset_x += 2;
        self.choices[index].render(&mut layout, b)?;

        b.set_fg(Color::Reset)
    }

    fn is_selectable(&self, index: usize) -> bool {
        (self.is_selectable)(&self.choices[index])
    }

    fn page_size(&self) -> usize {
        self.page_size
    }

    fn should_loop(&self) -> bool {
        self.should_loop
    }

    fn height_at(&mut self, index: usize, mut layout: ui::layout::Layout) -> u16 {
        layout.offset_x += 2;

        self[index].height(&mut layout)
    }

    fn len(&self) -> usize {
        self.choices.len()
    }
}

pub(crate) type ChoiceList<T> = SelectList<Choice<T>>;

impl<T> std::iter::FromIterator<T> for ChoiceList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new(Choice::is_choice);
        this.choices = iter.into_iter().map(Choice::Choice).collect();
        this
    }
}

impl<T> Default for ChoiceList<T> {
    fn default() -> Self {
        Self::new(Choice::is_choice)
    }
}

/// A possible choice in a list.
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Choice<T> {
    /// The main variant which represents a choice the user can pick from.
    Choice(T),
    /// A separator is a _single line_ of text that can be used to annotate other choices. It is not
    /// selectable and is skipped over when users navigate.
    ///
    /// If the text is more than one line, it will be cut-off.
    Separator(String),
    /// A separator which prints a line: "──────────────"
    DefaultSeparator,
}

impl<T> Choice<T> {
    /// Maps an `Choice<T>` to `Choice<U>` by applying a function to the contained
    /// [`Choice::Choice`] if any.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Choice<U> {
        match self {
            Choice::Choice(c) => Choice::Choice(f(c)),
            Choice::Separator(s) => Choice::Separator(s),
            Choice::DefaultSeparator => Choice::DefaultSeparator,
        }
    }

    /// Returns `true` if the choice is a [`Choice::Choice`].
    pub fn is_choice(&self) -> bool {
        matches!(self, Choice::Choice(_))
    }

    /// Returns `true` if the choice is a separator.
    pub fn is_separator(&self) -> bool {
        !self.is_choice()
    }

    /// Converts `&Choice<T>` to `Choice<&T>`.
    ///
    /// This will clone the [`Choice::Separator`] if any.
    pub fn as_ref(&self) -> Choice<&T> {
        match self {
            Choice::Choice(t) => Choice::Choice(t),
            Choice::Separator(s) => Choice::Separator(s.clone()),
            Choice::DefaultSeparator => Choice::DefaultSeparator,
        }
    }

    /// Converts `&mut Choice<T>` to `Choice<&mut T>`.
    ///
    /// This will clone the [`Choice::Separator`] if any.
    pub fn as_mut(&mut self) -> Choice<&mut T> {
        match self {
            Choice::Choice(t) => Choice::Choice(t),
            Choice::Separator(s) => Choice::Separator(s.clone()),
            Choice::DefaultSeparator => Choice::DefaultSeparator,
        }
    }

    /// Returns the contained [`Choice::Choice`] value, consuming `self`.
    ///
    /// # Panics
    ///
    /// This will panic if `self` is not a [`Choice::Choice`].
    pub fn unwrap_choice(self) -> T {
        match self {
            Choice::Choice(c) => c,
            _ => panic!("Called unwrap_choice on separator"),
        }
    }
}

#[inline]
pub(crate) fn get_sep_str<T>(separator: &Choice<T>) -> &str {
    match separator {
        Choice::Choice(_) => unreachable!(),
        Choice::Separator(s) => s,
        Choice::DefaultSeparator => "──────────────",
    }
}

impl<T: ui::Widget> ui::Widget for Choice<T> {
    fn render<B: ui::backend::Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        backend: &mut B,
    ) -> io::Result<()> {
        match self {
            Choice::Choice(c) => c.render(layout, backend),
            sep => get_sep_str(sep).render(layout, backend),
        }
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        match self {
            Choice::Choice(c) => c.height(layout),
            _ => 1,
        }
    }

    fn handle_key(&mut self, key: ui::events::KeyEvent) -> bool {
        match self {
            Choice::Choice(c) => c.handle_key(key),
            _ => false,
        }
    }

    /// This will panic if called
    fn cursor_pos(&mut self, _: ui::layout::Layout) -> (u16, u16) {
        unimplemented!("This should not be called")
    }
}

impl<I: Into<String>> From<I> for Choice<String> {
    fn from(s: I) -> Self {
        Choice::Choice(s.into())
    }
}

impl<I: Into<ExpandItem>> From<I> for Choice<ExpandItem> {
    fn from(item: I) -> Self {
        Choice::Choice(item.into())
    }
}

impl<I: Into<String>> From<(I, bool)> for Choice<(String, bool)> {
    fn from((text, checked): (I, bool)) -> Self {
        Choice::Choice((text.into(), checked))
    }
}

use std::ops::{Index, IndexMut};

use crate::ExpandItem;

#[derive(Debug)]
pub(crate) struct ChoiceList<T> {
    pub(crate) choices: Vec<Choice<T>>,
    page_size: usize,
    default: usize,
    // note: default is not an option usize because it adds an extra usize of space
    has_default: bool,
    should_loop: bool,
}

impl<T> ChoiceList<T> {
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

impl<T> Index<usize> for ChoiceList<T> {
    type Output = Choice<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.choices[index]
    }
}

impl<T> IndexMut<usize> for ChoiceList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.choices[index]
    }
}

impl<T> Default for ChoiceList<T> {
    fn default() -> Self {
        Self {
            choices: Vec::new(),
            page_size: 15,
            default: 0,
            has_default: false,
            should_loop: true,
        }
    }
}

#[derive(Debug)]
pub enum Choice<T> {
    Choice(T),
    Separator(String),
    DefaultSeparator,
}

impl<T> Choice<T> {
    pub(crate) fn is_separator(&self) -> bool {
        matches!(self, Choice::Separator(_) | Choice::DefaultSeparator)
    }

    pub(crate) fn as_ref(&self) -> Choice<&T> {
        match self {
            Choice::Choice(t) => Choice::Choice(t),
            Choice::Separator(s) => Choice::Separator(s.clone()),
            Choice::DefaultSeparator => Choice::DefaultSeparator,
        }
    }

    pub(crate) fn as_mut(&mut self) -> Choice<&mut T> {
        match self {
            Choice::Choice(t) => Choice::Choice(t),
            Choice::Separator(s) => Choice::Separator(s.clone()),
            Choice::DefaultSeparator => Choice::DefaultSeparator,
        }
    }

    pub(crate) fn unwrap_choice(self) -> T {
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
        layout: ui::Layout,
        backend: &mut B,
    ) -> ui::error::Result<()> {
        match self {
            Choice::Choice(c) => c.render(layout, backend),
            sep => get_sep_str(sep).render(layout, backend),
        }
    }

    fn height(&mut self, layout: ui::Layout) -> u16 {
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

    fn cursor_pos(&mut self, _: ui::Layout) -> (u16, u16) {
        unimplemented!("This should not be called")
    }
}

impl<T> From<T> for Choice<T> {
    fn from(t: T) -> Self {
        Choice::Choice(t)
    }
}

impl From<&'_ str> for Choice<String> {
    fn from(s: &str) -> Self {
        Choice::Choice(s.into())
    }
}

impl<I: Into<String>> From<(char, I)> for Choice<ExpandItem<String>> {
    fn from((key, name): (char, I)) -> Self {
        Choice::Choice(ExpandItem {
            key,
            name: name.into(),
        })
    }
}

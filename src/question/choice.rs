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
        self.has_default.then(|| self.default)
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
    Separator(Option<String>),
}

impl<T> Choice<T> {
    pub(crate) fn is_separator(&self) -> bool {
        matches!(self, Choice::Separator(_))
    }

    pub(crate) fn as_ref(&self) -> Choice<&T> {
        match self {
            Choice::Choice(t) => Choice::Choice(t),
            Choice::Separator(s) => Choice::Separator(s.clone()),
        }
    }

    pub(crate) fn unwrap_choice(self) -> T {
        if let Choice::Choice(c) = self {
            c
        } else {
            panic!("Called unwrap_choice on separator")
        }
    }
}

pub(crate) fn get_sep_str(separator: &Option<String>) -> &str {
    separator
        .as_ref()
        .map(String::as_str)
        .unwrap_or("──────────────")
}

impl<T: AsRef<str>> Choice<T> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Choice::Choice(t) => t.as_ref(),
            Choice::Separator(s) => get_sep_str(s),
        }
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

impl<I: Into<String>> From<(char, I)> for Choice<ExpandItem> {
    fn from((key, name): (char, I)) -> Self {
        Choice::Choice(ExpandItem {
            key,
            name: name.into(),
        })
    }
}

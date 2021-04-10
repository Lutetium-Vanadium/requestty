use std::ops::{Index, IndexMut};

pub(crate) struct ChoiceList<T> {
    pub(crate) choices: Vec<Choice<T>>,
    pub(crate) default: usize,
    pub(crate) should_loop: bool,
    pub(crate) page_size: usize,
}

impl<T> ChoiceList<T> {
    pub(crate) fn len(&self) -> usize {
        self.choices.len()
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

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
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

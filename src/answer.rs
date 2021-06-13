use std::{
    collections::hash_map::{Entry, IntoIter},
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "ahash")]
use ahash::AHashMap as HashMap;
#[cfg(not(feature = "ahash"))]
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Answer {
    String(String),
    ListItem(ListItem),
    ExpandItem(ExpandItem<String>),
    Int(i64),
    Float(f64),
    Bool(bool),
    ListItems(Vec<ListItem>),
}

impl Answer {
    /// Returns `true` if the answer is [`String`].
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn try_into_string(self) -> Result<String, Self> {
        match self {
            Self::String(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`ListItem`].
    pub fn is_list_item(&self) -> bool {
        matches!(self, Self::ListItem(..))
    }

    pub fn try_into_list_item(self) -> Result<ListItem, Self> {
        match self {
            Self::ListItem(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`ExpandItem`].
    pub fn is_expand_item(&self) -> bool {
        matches!(self, Self::ExpandItem(..))
    }

    pub fn try_into_expand_item(self) -> Result<ExpandItem<String>, Self> {
        match self {
            Self::ExpandItem(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Int`].
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..))
    }

    pub fn try_into_int(self) -> Result<i64, Self> {
        match self {
            Self::Int(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Float`].
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(..))
    }

    pub fn try_into_float(self) -> Result<f64, Self> {
        match self {
            Self::Float(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Bool`].
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(..))
    }

    pub fn try_into_bool(self) -> Result<bool, Self> {
        match self {
            Self::Bool(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`ListItems`].
    pub fn is_list_items(&self) -> bool {
        matches!(self, Self::ListItems(..))
    }

    pub fn try_into_list_items(self) -> Result<Vec<ListItem>, Self> {
        match self {
            Self::ListItems(v) => Ok(v),
            _ => Err(self),
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_list_item(&self) -> Option<&ListItem> {
        if let Self::ListItem(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_expand_item(&self) -> Option<&ExpandItem<String>> {
        if let Self::ExpandItem(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Self::Int(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Self::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_list_items(&self) -> Option<&Vec<ListItem>> {
        if let Self::ListItems(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListItem {
    pub index: usize,
    pub name: String,
}

impl From<(usize, String)> for ListItem {
    fn from((index, name): (usize, String)) -> Self {
        Self { index, name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpandItem<S> {
    pub key: char,
    pub name: S,
}

impl<I: Into<String>> From<(char, I)> for ExpandItem<String> {
    fn from((key, name): (char, I)) -> Self {
        Self {
            key,
            name: name.into(),
        }
    }
}

impl<S: ui::Widget> ui::Widget for ExpandItem<S> {
    fn render<B: ui::backend::Backend>(
        &mut self,
        layout: &mut ui::layout::Layout,
        backend: &mut B,
    ) -> ui::error::Result<()> {
        self.name.render(layout, backend)
    }

    fn height(&mut self, layout: &mut ui::layout::Layout) -> u16 {
        self.name.height(layout)
    }

    fn handle_key(&mut self, key: ui::events::KeyEvent) -> bool {
        self.name.handle_key(key)
    }

    fn cursor_pos(&mut self, layout: ui::layout::Layout) -> (u16, u16) {
        self.name.cursor_pos(layout)
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Answers {
    answers: HashMap<String, Answer>,
}

impl std::fmt::Debug for Answers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.answers.fmt(f)
    }
}

impl Answers {
    pub(crate) fn insert(&mut self, name: String, answer: Answer) -> &mut Answer {
        match self.answers.entry(name) {
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                *entry = answer;
                entry
            }
            Entry::Vacant(entry) => entry.insert(answer),
        }
    }
}

impl Extend<(String, Answer)> for Answers {
    fn extend<T: IntoIterator<Item = (String, Answer)>>(&mut self, iter: T) {
        self.answers.extend(iter)
    }

    #[cfg(nightly)]
    fn extend_one(&mut self, item: (String, Answer)) {
        self.answers.extend_one(item);
    }

    #[cfg(nightly)]
    fn extend_reserve(&mut self, additional: usize) {
        self.answers.extend_reserve(additional)
    }
}

impl Deref for Answers {
    type Target = HashMap<String, Answer>;

    fn deref(&self) -> &Self::Target {
        &self.answers
    }
}

impl DerefMut for Answers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.answers
    }
}

impl IntoIterator for Answers {
    type Item = (String, Answer);
    type IntoIter = IntoIter<String, Answer>;

    fn into_iter(self) -> Self::IntoIter {
        self.answers.into_iter()
    }
}

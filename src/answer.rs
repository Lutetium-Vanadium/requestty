use std::{
    collections::hash_map::{Entry, HashMap, IntoIter},
    hash::Hash,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

/// The different answer types that can be returned by the [`Question`]s
///
/// [`Question`]: crate::question::Question
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Answer {
    /// Strings will be returned by [`input`], [`password`] and [`editor`].
    ///
    /// [`input`]: crate::question::Question::input
    /// [`password`]: crate::question::Question::password
    /// [`editor`]: crate::question::Question::editor
    String(String),
    /// ListItems will be returned by [`select`] and [`raw_select`].
    ///
    /// [`select`]: crate::question::Question::select
    /// [`raw_select`]: crate::question::Question::raw_select
    ListItem(ListItem),
    /// ExpandItems will be returned by [`expand`].
    ///
    /// [`expand`]: crate::question::Question::expand
    ExpandItem(ExpandItem),
    /// Ints will be returned by [`int`].
    ///
    /// [`int`]: crate::question::Question::int
    Int(i64),
    /// Floats will be returned by [`float`].
    ///
    /// [`float`]: crate::question::Question::float
    Float(f64),
    /// Bools will be returned by [`confirm`].
    ///
    /// [`confirm`]: crate::question::Question::confirm
    Bool(bool),
    /// ListItems will be returned by [`multi_select`].
    ///
    /// [`multi_select`]: crate::question::Question::multi_select
    ListItems(Vec<ListItem>),
}

impl Answer {
    /// Returns `true` if the answer is [`Answer::String`].
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    /// Returns [`Some`] if it is a [`Answer::String`], otherwise returns [`None`].
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the `Ok(String)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_string(self) -> Result<String, Self> {
        match self {
            Self::String(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Answer::ListItem`].
    pub fn is_list_item(&self) -> bool {
        matches!(self, Self::ListItem(..))
    }

    /// Returns [`Some`] if it is a [`Answer::ListItem`], otherwise returns [`None`].
    pub fn as_list_item(&self) -> Option<&ListItem> {
        match self {
            Self::ListItem(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the `Ok(ListItem)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_list_item(self) -> Result<ListItem, Self> {
        match self {
            Self::ListItem(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Answer::ExpandItem`].
    pub fn is_expand_item(&self) -> bool {
        matches!(self, Self::ExpandItem(..))
    }

    /// Returns [`Some`] if it is [`Answer::ExpandItem`], otherwise returns [`None`].
    pub fn as_expand_item(&self) -> Option<&ExpandItem> {
        match self {
            Self::ExpandItem(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the `Ok(ExpandItem)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_expand_item(self) -> Result<ExpandItem, Self> {
        match self {
            Self::ExpandItem(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Answer::Int`].
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(..))
    }

    /// Returns [`Some`] if it is [`Answer::Int`], otherwise returns [`None`].
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the `Ok(i64)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_int(self) -> Result<i64, Self> {
        match self {
            Self::Int(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Answer::Float`].
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(..))
    }

    /// Returns [`Some`] if it is [`Answer::Float`], otherwise returns [`None`].
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the `Ok(f64)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_float(self) -> Result<f64, Self> {
        match self {
            Self::Float(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Answer::Bool`].
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(..))
    }

    /// Returns [`Some`] if it is [`Answer::Bool`], otherwise returns [`None`].
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the `Ok(bool)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_bool(self) -> Result<bool, Self> {
        match self {
            Self::Bool(v) => Ok(v),
            _ => Err(self),
        }
    }

    /// Returns `true` if the answer is [`Answer::ListItems`].
    pub fn is_list_items(&self) -> bool {
        matches!(self, Self::ListItems(..))
    }

    /// Returns [`Some`] if it is [`Answer::ListItems`], otherwise returns [`None`].
    pub fn as_list_items(&self) -> Option<&[ListItem]> {
        match self {
            Self::ListItems(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the `Ok(Vec<ListItem>)` if it is one, otherwise returns itself as an [`Err`].
    pub fn try_into_list_items(self) -> Result<Vec<ListItem>, Self> {
        match self {
            Self::ListItems(v) => Ok(v),
            _ => Err(self),
        }
    }
}

/// A representation of a [`Choice`] at a particular index.
///
/// It will be returned by [`select`] and [`raw_select`].
///
/// [`Choice`]: crate::Choice
/// [`select`]: crate::question::Question::select
/// [`raw_select`]: crate::question::Question::raw_select
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListItem {
    /// The index of the choice
    pub index: usize,
    /// The content of the choice -- it is what was displayed to the user
    pub text: String,
}

impl<I: Into<String>> From<(usize, I)> for ListItem {
    fn from((index, text): (usize, I)) -> Self {
        Self {
            index,
            text: text.into(),
        }
    }
}

/// A representation of a [`Choice`] for a particular key.
///
/// It will be returned by [`expand`].
///
/// [`Choice`]: crate::Choice
/// [`expand`]: crate::question::Question::expand
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpandItem {
    /// The key associated with the choice
    pub key: char,
    /// The content of the choice -- it is what was displayed to the user
    pub text: String,
}

impl<I: Into<String>> From<(char, I)> for ExpandItem {
    fn from((key, text): (char, I)) -> Self {
        Self {
            key,
            text: text.into(),
        }
    }
}

/// A collections of answers of previously asked [`Question`]s.
///
/// [`Question`]: crate::question::Question
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

impl From<HashMap<String, Answer>> for Answers {
    fn from(answers: HashMap<String, Answer>) -> Self {
        Self { answers }
    }
}

impl FromIterator<(String, Answer)> for Answers {
    fn from_iter<T: IntoIterator<Item = (String, Answer)>>(iter: T) -> Self {
        Self {
            answers: iter.into_iter().collect(),
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

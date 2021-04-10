use fxhash::FxHashSet as HashSet;

#[derive(Debug, Clone)]
pub enum Answer {
    String(String),
    ListItem(ListItem),
    ExpandItem(ExpandItem),
    Int(i64),
    Float(f64),
    Bool(bool),
    ListItems(HashSet<ListItem>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListItem {
    pub index: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpandItem {
    pub key: char,
    pub name: String,
}

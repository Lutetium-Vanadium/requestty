//! Special characters used for prompts/widgets.
//!
//! There are 2 default [`SymbolSet`]s -- [`UNICODE`] and [`ASCII`]. If a particular [`SymbolSet`]
//! is not set, [`UNICODE`] is used. The [`ASCII`] symbol set exists if you want to have larger
//! compatibility with terminal emulators (such as Windows' `cmd.exe`) which do not support unicode
//! characters.

use std::sync::Mutex;

use once_cell::sync::Lazy;

static SET: Lazy<Mutex<SymbolSet>> = Lazy::new(|| Mutex::new(UNICODE));

/// Get the current [`SymbolSet`]
///
/// If not set, it defaults to the [`UNICODE`] symbol set.
///
/// Also see [`symbols::set`](set).
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "ignore this line for doc test as requestty_ui should be used")]
/// use requestty::symbols;
/// # use requestty_ui::symbols;
///
/// let symbol_set = symbols::current();
/// println!("{}", symbol_set.pointer);
/// ```
pub fn current() -> SymbolSet {
    SET.lock().expect("symbol set poisoned").clone()
}

/// Set the current [`SymbolSet`]
///
/// Also see [`symbols::current`](current).
///
/// # Example
///
/// ```
/// # #[cfg(feature = "ignore this line for doc test as requestty_ui should be used")]
/// use requestty::symbols;
/// # use requestty_ui::symbols;
///
/// symbols::set(symbols::ASCII);
/// assert_eq!(symbols::current(), symbols::ASCII);
/// ```
pub fn set(new: SymbolSet) {
    *SET.lock().expect("symbol set poisoned") = new;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The various special symbols used by the prompts during rendering.
pub struct SymbolSet {
    /// Used to point to a special item.
    ///
    /// For example, this is used in the various list prompts to show the currently hovered item.
    pub pointer: char,
    /// Generic decoration mark which points to the right.
    ///
    /// For example, this is used in the prompts when there is no hint between the question and the
    /// answer.
    pub arrow: char,
    /// Decoration to show when a question is completed.
    ///
    /// For example, this is replaces the question mark when the question is answered.
    pub completed: char,
    /// Decoration to add some spacing without leaving it empty.
    ///
    /// For example, this is used by prompts to separate an answered question from its answer.
    pub middle_dot: char,
    /// A symbol to indicate something being wrong.
    ///
    /// For example, this is used by prompts if validation fails.
    pub cross: char,
    /// Character for the top right corner of a box.
    pub box_top_right: char,
    /// Character for the top left corner of a box.
    pub box_top_left: char,
    /// Character for the bottom right corner of a box.
    pub box_bottom_right: char,
    /// Character for the bottom left corner of a box.
    pub box_bottom_left: char,
    /// Character for the horizontal edge of a box.
    pub box_horizontal: char,
    /// Character for the vertical edge of a box.
    pub box_vertical: char,
}

/// The default [`SymbolSet`].
///
/// It is composed of unicode characters and so may not be supported by all terminal emulators.
pub const UNICODE: SymbolSet = SymbolSet {
    /// `'❯' U+276F`
    pointer: '❯',
    /// `'›' U+203A`
    arrow: '›',
    /// `'✔' U+2714`
    completed: '✔',
    /// `'·' U+00B7`
    middle_dot: '·',
    /// `'✖' U+2716`
    cross: '✖',
    /// `'┐' U+2510`
    box_top_right: '┐',
    /// `'┌' U+250C`
    box_top_left: '┌',
    /// `'┘' U+2518`
    box_bottom_right: '┘',
    /// `'└' U+2514`
    box_bottom_left: '└',
    /// `'─' U+2500`
    box_horizontal: '─',
    /// `'│' U+2502`
    box_vertical: '│',
};

/// A [`SymbolSet`] based exclusively on ASCII characters.
///
/// Since it contains only ASCII, it will be supported by all terminal emulators but may not look as
/// good.
pub const ASCII: SymbolSet = SymbolSet {
    pointer: '>',
    arrow: '>',
    completed: '?',
    middle_dot: '~',
    cross: 'x',
    box_top_right: '.',
    box_top_left: '.',
    box_bottom_right: '\'',
    box_bottom_left: '\'',
    box_horizontal: '-',
    box_vertical: '|',
};

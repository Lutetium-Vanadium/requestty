//! A widget based terminal ui rendering library.
//!
//! This crate provides an abstraction over terminal manipulation in the form of the [`Widget`]
//! trait. It also provides some default widgets available in [`widgets`].
//!
//! While this crate was built for the  [`discourse`] crate and other crates which implement the
//! `Plugin` trait in [`discourse`], it can be used otherwise as well.
//!
//! [`discourse`]: https://github.com/lutetium-vanadium/discourse
//!
//! # Backends
//!
//! This crate currently supports 2 backends:
//! - [`crossterm`](https://crates.io/crates/crossterm) (default)
//! - [`termion`](https://crates.io/crates/termion)
//!
//! The default backend is `crossterm` for the following reasons:
//! - Wider terminal support
//! - Better event processing (in my experience)
//!
//! The different backends can be enabled using the features of the same name.
// TODO: [`discourse`]: https://crates.io/crates/discourse
#![deny(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    broken_intra_doc_links
)]
#![warn(rust_2018_idioms)]

pub use error::{ErrorKind, Result};
pub use input::{Input, Prompt, Validation};
pub use widget::Widget;

/// A module containing the in-built widgets and types required by them
pub mod widgets {
    pub use crate::char_input::CharInput;
    pub use crate::prompt::{Delimiter, Prompt};
    pub use crate::select::{List, Select};
    pub use crate::string_input::StringInput;
    pub use crate::text::Text;
    pub use crate::widget::Widget;

    /// The default type for `filter_map` in [`StringInput`] and [`CharInput`]
    pub type FilterMapChar = fn(char) -> Option<char>;

    /// Character filter that lets every character through
    pub(crate) fn no_filter(c: char) -> Option<char> {
        Some(c)
    }
}

pub mod backend;
mod char_input;
mod error;
pub mod events;
mod input;
pub mod layout;
mod prompt;
mod select;
mod string_input;
pub mod style;
mod text;
mod widget;

/// Some characters used in the `discourse` crate.
#[allow(missing_docs)]
pub mod symbols {
    /// `'❯' U+276F`
    pub const ARROW: char = '❯';
    /// `'›' U+203A`
    pub const SMALL_ARROW: char = '›';
    /// `'✔' U+2714`
    pub const TICK: char = '✔';
    /// `'·' U+00B7`
    pub const MIDDLE_DOT: char = '·';
    /// `'✖' U+2716`
    pub const CROSS: char = '✖';
    /// `'┐' U+2510`
    pub const BOX_LIGHT_TOP_RIGHT: char = '┐';
    /// `'┌' U+250C`
    pub const BOX_LIGHT_TOP_LEFT: char = '┌';
    /// `'┘' U+2518`
    pub const BOX_LIGHT_BOTTOM_RIGHT: char = '┘';
    /// `'└' U+2514`
    pub const BOX_LIGHT_BOTTOM_LEFT: char = '└';
    /// `'─' U+2500`
    pub const BOX_LIGHT_HORIZONTAL: char = '─';
    /// `'│' U+2502`
    pub const BOX_LIGHT_VERTICAL: char = '│';
}

#[doc(hidden)]
pub mod features {
    #[cfg(feature = "crossterm")]
    pub const SNAPSHOT_PATH: &str = "crossterm-snapshots";

    #[cfg(feature = "termion")]
    pub const SNAPSHOT_PATH: &str = "termion-snapshots";
}

/// A testing utility to assert visual equality with [`TestBackend`](backend::TestBackend).
///
/// It is a simple wrapper around [`insta::assert_display_snapshot`] which puts the snapshots in
/// `$CARGO_MANIFEST_DIR/{crossterm/termion}-snapshots`.
///
/// [`insta::assert_display_snapshot`]: https://docs.rs/insta/1.7.1/insta/macro.assert_display_snapshot.html
#[macro_export]
macro_rules! assert_backend_snapshot {
    ($value:expr, @$snapshot:literal) => {
        $crate::assert_backend_snapshot_impl!(::insta::assert_display_snapshot!($value, @$snapshot))
    };
    ($name:expr, $value:expr) => {
        $crate::assert_backend_snapshot_impl!(::insta::assert_display_snapshot!($name, $value))
    };
    ($value:expr) => {
        $crate::assert_backend_snapshot_impl!(::insta::assert_display_snapshot!($value))
    };

}

#[doc(hidden)]
#[macro_export]
macro_rules! assert_backend_snapshot_impl {
    ($($tt:tt)*) => {{
        ::insta::with_settings!({
            snapshot_path => ::std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join($crate::features::SNAPSHOT_PATH)
        }, {
            $($tt)*
        })
    }}
}

#[cfg(test)]
mod test_consts {
    /// ASCII placeholder text with 470 characters
    pub(crate) static LOREM: &str = "Lorem ipsum dolor sit amet, consectetuer \
        adipiscing elit. Aenean commodo ligula e get dolor. Aenean massa. Cum sociis \
        natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla \
        consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, \
        vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, \
        justo. Nullam dictum felis eu pede mollis pretium.";

    /// Unicode placeholder text with 470 characters
    pub(crate) static UNICODE: &str = "ǹɕǶǽũ ȥűǷŀȷÂǦǨÏǊ ýǡƎƭǃÁžƖţŝŬœĶ ɳƙŁŵŃŋŗ ǳÆŅɜŴô ħĲǗɧÝÙĝɸÿ \
        ǝƬǄƫɌñÄç ɎƷɔȲƧ éďŅǒƿŅ üĲƪɮúǚĳǓɔÏǙǟ ǃóıÄ×ȤøŌɘŬÂ ȃŜʈǑƱļ ȶė÷ƝȣŞýş óɭǽƎȮ ŏŀƔȾřŞȩ \
        ĚïƝƦʀƕĥǡǎÌʅ ĻɠȞīĈưĭÓĢÑ ǇĦƷűǐ¾đ ŊǂȘŰƒ ēɄɟɍƬč ɼ·ȄĶȸŦɉ ţĥŐŉŭ ãɹƠƲɼŒǜ ȹúƄǆȆ ȡǞȐǖŁƀ \
        ėýŭȇȹı ɹûØùž ïȕĆßĀȭ ÍȖȟũȍ ȼƦŚɀʆ ĖǱŞȅŎ ţÎǓŏï ȃāÖćźȀȿ Īŝłƒťƌȇ ǘůńǊļ ǂȄȐǐǻ Ȳɵ¾ǕÉ \
        ɛȃǾȚǱÚ ķĘƄɜÉ êɷƐŻɌ ɐțǼÏƐȄ òɫɥƸâɈ ĄȫĞîĖƿſú¹ ǐȊÜÉį ȬǲɩŎǩĮ ĂȷĎǶŐ ÍɼƔÌűÉĎƣ ÃÜȯƪǇ \
        ȋǲŹǀŊȻ Ɍ¾ȓƃĝ êɊǄɕÈ ÿ¸ȧȣíÚɁƺ ȏǖŷȡȬ ȍǕȁɜğʆ ƨɺȨƠŇȱƕ ȊÑļɧģŷĲ ʈźçƣƑ ƀǼŌéǔÀ ȊŅɂƵǝ¾ \
        ēɩīűŃɖąɔɳ ȁõıĚņ ȦɂȄƄȥɣŴűǎǃ";
}

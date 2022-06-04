//! A widget based terminal ui rendering library.
//!
//! This crate provides an abstraction over terminal manipulation in the form of the [`Widget`]
//! trait. It also provides some default widgets available in [`widgets`].
//!
//! While this crate was built for the  [`requestty`] crate and other crates which implement the
//! [`Prompt`] trait in [`requestty`], it can be used otherwise as well.
//!
//! [`requestty`]: https://crates.io/crates/requestty
//! [`Prompt`]: https://docs.rs/requestty/latest/requestty/prompt/trait.Prompt.html
//!
//! # Backends
//!
//! This crate currently supports 2 backends:
//! - [`crossterm`](https://crates.io/crates/crossterm)
//! - [`termion`](https://crates.io/crates/termion)
//!
//! The different backends can be enabled using the features of the same name.
#![deny(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    rustdoc::broken_intra_doc_links
)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use error::{ErrorKind, Result};
pub use input::{Input, OnEsc, Prompt, Validation};
pub use widgets::Widget;

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
pub mod symbols;
mod text;
pub mod widgets;

#[doc(hidden)]
pub mod features {
    #[cfg(feature = "crossterm")]
    pub const SNAPSHOT_PATH: &str = "crossterm-snapshots";

    // XXX: Only works when crossterm and termion are the only two available backends
    //
    // Instead of directly checking for termion, we check for not crossterm so that compiling
    // (documentation) with both features enabled will not error
    #[cfg(not(feature = "crossterm"))]
    pub const SNAPSHOT_PATH: &str = "termion-snapshots";
}

/// A testing utility to assert visual equality with [`TestBackend`](backend::TestBackend).
///
/// It is a simple wrapper around [`insta::assert_display_snapshot`] which puts the snapshots in
/// `$CARGO_MANIFEST_DIR/{crossterm/termion}-snapshots`.
///
/// [`insta::assert_display_snapshot`]: https://docs.rs/insta/1.11.0/insta/macro.assert_display_snapshot.html
#[cfg(any(feature = "crossterm", feature = "termion"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "crossterm", feature = "termion"))))]
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

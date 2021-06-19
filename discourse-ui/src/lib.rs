//! A widget based cli ui rendering library
pub use input::{Input, Prompt};
pub use widget::Widget;

/// In build widgets
pub mod widgets {
    pub use crate::char_input::CharInput;
    pub use crate::prompt::{Delimiter, Prompt};
    pub use crate::select::{List, Select};
    pub use crate::string_input::StringInput;
    pub use crate::text::Text;
    pub use crate::widget::Widget;

    /// The default type for filter_map_char in [`StringInput`] and [`CharInput`]
    pub type FilterMapChar = fn(char) -> Option<char>;

    /// Character filter that lets every character through
    pub fn no_filter(c: char) -> Option<char> {
        Some(c)
    }
}

pub mod backend;
mod char_input;
pub mod error;
pub mod events;
mod input;
pub mod layout;
mod prompt;
mod select;
mod string_input;
pub mod style;
mod text;
mod widget;

/// Returned by [`Prompt::validate`]
pub enum Validation {
    /// If the prompt is ready to finish.
    Finish,
    /// If the state is valid, but the prompt should still persist.
    /// Unlike returning an Err, this will not print anything unique, and is a way for the prompt to
    /// say that it internally has processed the `Enter` key, but is not complete.
    Continue,
}

pub mod symbols {
    pub const ARROW: char = '❯';
    pub const SMALL_ARROW: char = '›';
    pub const TICK: char = '✔';
    pub const MIDDLE_DOT: char = '·';
    pub const CROSS: char = '✖';
    pub const BOX_LIGHT_TOP_RIGHT: char = '┐';
    pub const BOX_LIGHT_TOP_LEFT: char = '┌';
    pub const BOX_LIGHT_BOTTOM_RIGHT: char = '┘';
    pub const BOX_LIGHT_BOTTOM_LEFT: char = '└';
    pub const BOX_LIGHT_HORIZONTAL: char = '─';
    pub const BOX_LIGHT_VERTICAL: char = '│';
}

#[doc(hidden)]
pub mod features {
    #[cfg(feature = "crossterm")]
    pub const SNAPSHOT_PATH: &str = "crossterm-snapshots";

    #[cfg(feature = "termion")]
    pub const SNAPSHOT_PATH: &str = "termion-snapshots";
}

#[macro_export]
macro_rules! assert_backend_snapshot {
    ($value:expr, @$snapshot:literal) => {
        $crate::assert_backend_snapshot!(@__impl ::insta::assert_display_snapshot!($value, @$snapshot))
    };
    ($name:expr, $value:expr) => {
        $crate::assert_backend_snapshot!(@__impl ::insta::assert_display_snapshot!($name, $value))
    };
    ($value:expr) => {
        $crate::assert_backend_snapshot!(@__impl ::insta::assert_display_snapshot!($value))
    };

    (@__impl $($tt:tt)*) => {{
        ::insta::with_settings!({
            snapshot_path => ::std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join($crate::features::SNAPSHOT_PATH)
        }, {
            $($tt)*
        })
    }
    }
}

#[cfg(test)]
mod test_consts {
    /// ASCII placeholder text with 470 characters
    pub static LOREM: &str = "Lorem ipsum dolor sit amet, consectetuer \
        adipiscing elit. Aenean commodo ligula e get dolor. Aenean massa. Cum sociis \
        natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla \
        consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, \
        vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, \
        justo. Nullam dictum felis eu pede mollis pretium.";

    /// Unicode placeholder text with 470 characters
    pub static UNICODE: &str = "ǹɕǶǽũ ȥűǷŀȷÂǦǨÏǊ ýǡƎƭǃÁžƖţŝŬœĶ ɳƙŁŵŃŋŗ ǳÆŅɜŴô ħĲǗɧÝÙĝɸÿ \
        ǝƬǄƫɌñÄç ɎƷɔȲƧ éďŅǒƿŅ üĲƪɮúǚĳǓɔÏǙǟ ǃóıÄ×ȤøŌɘŬÂ ȃŜʈǑƱļ ȶė÷ƝȣŞýş óɭǽƎȮ ŏŀƔȾřŞȩ \
        ĚïƝƦʀƕĥǡǎÌʅ ĻɠȞīĈưĭÓĢÑ ǇĦƷűǐ¾đ ŊǂȘŰƒ ēɄɟɍƬč ɼ·ȄĶȸŦɉ ţĥŐŉŭ ãɹƠƲɼŒǜ ȹúƄǆȆ ȡǞȐǖŁƀ \
        ėýŭȇȹı ɹûØùž ïȕĆßĀȭ ÍȖȟũȍ ȼƦŚɀʆ ĖǱŞȅŎ ţÎǓŏï ȃāÖćźȀȿ Īŝłƒťƌȇ ǘůńǊļ ǂȄȐǐǻ Ȳɵ¾ǕÉ \
        ɛȃǾȚǱÚ ķĘƄɜÉ êɷƐŻɌ ɐțǼÏƐȄ òɫɥƸâɈ ĄȫĞîĖƿſú¹ ǐȊÜÉį ȬǲɩŎǩĮ ĂȷĎǶŐ ÍɼƔÌűÉĎƣ ÃÜȯƪǇ \
        ȋǲŹǀŊȻ Ɍ¾ȓƃĝ êɊǄɕÈ ÿ¸ȧȣíÚɁƺ ȏǖŷȡȬ ȍǕȁɜğʆ ƨɺȨƠŇȱƕ ȊÑļɧģŷĲ ʈźçƣƑ ƀǼŌéǔÀ ȊŅɂƵǝ¾ \
        ēɩīűŃɖąɔɳ ȁõıĚņ ȦɂȄƄȥɣŴűǎǃ";
}

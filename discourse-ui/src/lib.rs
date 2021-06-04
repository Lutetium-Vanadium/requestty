//! A widget based cli ui rendering library
use std::ops::{Deref, DerefMut};

pub use input::{Input, Prompt};
pub use widget::Widget;

/// In build widgets
pub mod widgets {
    pub use super::char_input::CharInput;
    pub use super::select::{List, Select};
    pub use super::string_input::StringInput;
    pub use super::text::Text;

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
mod select;
mod string_input;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The part of the text to render if the full text cannot be rendered
pub enum RenderRegion {
    Top,
    Middle,
    Bottom,
}

impl Default for RenderRegion {
    fn default() -> Self {
        RenderRegion::Middle
    }
}

/// Assume the highlighted part of the block below is the place available for rendering
/// in the given box
/// ```text
///  ____________
/// |            |
/// |     ███████|
/// |  ██████████|
/// |  ██████████|
/// '------------'
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct Layout {
    /// ```text
    ///  ____________
    /// |  vvv-- line_offset
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub line_offset: u16,
    /// ```text
    ///  ____________
    /// |vv-- offset_x
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub offset_x: u16,
    /// ```text
    ///  .-- offset_y
    /// |'>          |
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub offset_y: u16,
    /// ```text
    ///  ____________
    /// |            |
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    ///  ^^^^^^^^^^^^-- width
    /// ```
    pub width: u16,
    /// ```text
    ///  _____ height --.
    /// |            | <'
    /// |     ███████| <'
    /// |  ██████████| <'
    /// |  ██████████| <'
    /// '------------'
    /// ```
    pub height: u16,
    /// ```text
    ///  ____________
    /// |.-- max_height
    /// |'>   ███████|
    /// |'>██████████|
    /// |'>██████████|
    /// '------------'
    /// ```
    pub max_height: u16,
    /// The region to render if full text cannot be rendered
    pub render_region: RenderRegion,
}

impl Layout {
    pub fn new(line_offset: u16, size: backend::Size) -> Self {
        Self {
            line_offset,
            offset_x: 0,
            offset_y: 0,
            width: size.width,
            height: size.height,
            max_height: size.height,
            render_region: RenderRegion::Top,
        }
    }

    pub fn with_line_offset(mut self, line_offset: u16) -> Self {
        self.line_offset = line_offset;
        self
    }

    pub fn with_size(mut self, size: backend::Size) -> Self {
        self.set_size(size);
        self
    }

    pub fn with_offset(mut self, offset_x: u16, offset_y: u16) -> Self {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
        self
    }

    pub fn with_render_region(mut self, region: RenderRegion) -> Self {
        self.render_region = region;
        self
    }

    pub fn with_max_height(mut self, max_height: u16) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn set_size(&mut self, terminal_size: backend::Size) {
        self.width = terminal_size.width;
        self.height = terminal_size.height;
    }

    pub fn line_width(&self) -> u16 {
        self.width - self.line_offset - self.offset_x
    }

    pub fn get_start(&self, height: u16) -> u16 {
        if height > self.max_height {
            match self.render_region {
                RenderRegion::Top => 0,
                RenderRegion::Middle => (height - self.max_height) / 2,
                RenderRegion::Bottom => height - self.max_height,
            }
        } else {
            0
        }
    }
}

#[test]
fn test_layout() {
    let layout = Layout::new(0, (100, 5).into());
    assert_eq!(
        layout.with_render_region(RenderRegion::Top).get_start(10),
        0
    );
    assert_eq!(
        layout
            .with_render_region(RenderRegion::Middle)
            .get_start(10),
        2
    );
    assert_eq!(
        layout
            .with_render_region(RenderRegion::Bottom)
            .get_start(10),
        5
    );
}

struct TerminalState<B: backend::Backend> {
    backend: B,
    hide_cursor: bool,
    enabled: bool,
}

impl<B: backend::Backend> TerminalState<B> {
    fn new(backend: B, hide_cursor: bool) -> Self {
        Self {
            backend,
            enabled: false,
            hide_cursor,
        }
    }

    fn init(&mut self) -> error::Result<()> {
        self.enabled = true;
        if self.hide_cursor {
            self.backend.hide_cursor()?;
        }
        self.backend.enable_raw_mode()
    }

    fn reset(&mut self) -> error::Result<()> {
        self.enabled = false;
        if self.hide_cursor {
            self.backend.show_cursor()?;
        }
        self.backend.disable_raw_mode()
    }
}

impl<B: backend::Backend> Drop for TerminalState<B> {
    fn drop(&mut self) {
        if self.enabled {
            let _ = self.reset();
        }
    }
}

impl<B: backend::Backend> Deref for TerminalState<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: backend::Backend> DerefMut for TerminalState<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
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

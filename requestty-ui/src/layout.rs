//! A module to describe regions of the screen that can be rendered to.

/// The part of the text to render if the full text cannot be rendered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
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

/// `Layout` represents a portion of the screen that is available to be rendered to.
///
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
    /// Creates a new `Layout`.
    pub fn new(line_offset: u16, size: crate::backend::Size) -> Self {
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

    /// Creates a new `Layout` with given `line_offset`.
    pub fn with_line_offset(mut self, line_offset: u16) -> Self {
        self.line_offset = line_offset;
        self
    }

    /// Creates a new `Layout` with given `width` and `height`.
    pub fn with_size(mut self, size: crate::backend::Size) -> Self {
        self.set_size(size);
        self
    }

    /// Creates a new `Layout` with new `offset_x` and `offset_y`.
    pub fn with_offset(mut self, offset_x: u16, offset_y: u16) -> Self {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
        self
    }

    /// Creates a new `Layout` with new `render_region`.
    pub fn with_render_region(mut self, region: RenderRegion) -> Self {
        self.render_region = region;
        self
    }

    /// Creates a new `Layout` with new `max_height`.
    pub fn with_max_height(mut self, max_height: u16) -> Self {
        self.max_height = max_height;
        self
    }

    /// Creates a new `Layout` that represents a region past the `cursor_pos`. `cursor_pos` is
    /// relative to `offset_x` and `offset_y`.
    pub fn with_cursor_pos(mut self, cursor_pos: (u16, u16)) -> Self {
        self.line_offset = cursor_pos.0;
        // TODO: change to `=` if cursor_pos becomes absolute value
        self.offset_y += cursor_pos.1;
        self
    }

    /// Sets the `width` and `height` of the layout.
    pub fn set_size(&mut self, terminal_size: crate::backend::Size) {
        self.width = terminal_size.width;
        self.height = terminal_size.height;
    }

    /// Gets the width of renderable space on the first line.
    ///
    /// ```text
    ///  ____________
    /// |     vvvvvvv-- line_width
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub fn line_width(&self) -> u16 {
        self.available_width() - self.line_offset
    }

    /// Gets the width of renderable space on subsequent lines.
    ///
    /// ```text
    ///  ____________
    /// |  vvvvvvvvvv-- available_width
    /// |     ███████|
    /// |  ██████████|
    /// |  ██████████|
    /// '------------'
    /// ```
    pub fn available_width(&self) -> u16 {
        self.width - self.offset_x
    }

    /// Gets the starting line number for the given `height` taking into account the `max_height`
    /// and the `render_region`.
    ///
    /// If the height of the widget to render is 5 and the max_height is 2, then the start would be:
    /// - `RenderRegion::Top`: 0
    /// - `RenderRegion::Middle`: 1
    /// - `RenderRegion::Top`: 3
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

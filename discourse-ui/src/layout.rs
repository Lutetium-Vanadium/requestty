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

    pub fn with_line_offset(mut self, line_offset: u16) -> Self {
        self.line_offset = line_offset;
        self
    }

    pub fn with_size(mut self, size: crate::backend::Size) -> Self {
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

    pub fn with_cursor_pos(mut self, cursor_pos: (u16, u16)) -> Self {
        self.line_offset = cursor_pos.0;
        // TODO: change to `=` if cursor_pos becomes absolute value
        self.offset_y += cursor_pos.1;
        self
    }

    pub fn set_size(&mut self, terminal_size: crate::backend::Size) {
        self.width = terminal_size.width;
        self.height = terminal_size.height;
    }

    pub fn line_width(&self) -> u16 {
        self.available_width() - self.line_offset
    }

    pub fn available_width(&self) -> u16 {
        self.width - self.offset_x
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

use std::io;

use ui::{backend, style};

pub struct SnapshotOnFlushBackend {
    backend: backend::TestBackend,
}

impl SnapshotOnFlushBackend {
    #[allow(dead_code)]
    pub fn new(size: backend::Size) -> Self {
        Self {
            backend: backend::TestBackend::new(size),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_layout(size: backend::Size, layout: ui::layout::Layout) -> Self {
        Self {
            backend: backend::TestBackend::new_with_layout(size, layout),
        }
    }
}

impl io::Write for SnapshotOnFlushBackend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.backend.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        ui::assert_backend_snapshot!(self.backend);
        Ok(())
    }
}

impl backend::Backend for SnapshotOnFlushBackend {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        self.backend.enable_raw_mode()
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        self.backend.disable_raw_mode()
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.backend.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.backend.show_cursor()
    }

    fn get_cursor_pos(&mut self) -> io::Result<(u16, u16)> {
        self.backend.get_cursor_pos()
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.backend.move_cursor_to(x, y)
    }

    fn move_cursor(&mut self, direction: backend::MoveDirection) -> io::Result<()> {
        self.backend.move_cursor(direction)
    }

    fn scroll(&mut self, dist: i16) -> io::Result<()> {
        self.backend.scroll(dist)
    }

    fn set_attributes(&mut self, attributes: style::Attributes) -> io::Result<()> {
        self.backend.set_attributes(attributes)
    }

    fn set_fg(&mut self, color: style::Color) -> io::Result<()> {
        self.backend.set_fg(color)
    }

    fn set_bg(&mut self, color: style::Color) -> io::Result<()> {
        self.backend.set_bg(color)
    }

    fn clear(&mut self, clear_type: backend::ClearType) -> io::Result<()> {
        self.backend.clear(clear_type)
    }

    fn size(&self) -> io::Result<backend::Size> {
        self.backend.size()
    }
}

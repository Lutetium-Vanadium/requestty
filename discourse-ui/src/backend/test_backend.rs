use std::{fmt, io::Write};

use super::{Attributes, Backend, ClearType, Color, MoveDirection, Size as BSize};
use crate::error;
use TestBackendOp::*;

#[derive(Debug, Clone)]
pub enum TestBackendOp {
    Write(Vec<u8>),
    Flush,
    EnableRawMode,
    DisableRawMode,
    HideCursor,
    ShowCursor,
    /// The value to return
    GetCursor(u16, u16),
    SetCursor(u16, u16),
    MoveCursor(MoveDirection),
    Scroll(i16),
    SetAttributes(Attributes),
    RemoveAttributes(Attributes),
    SetFg(Color),
    SetBg(Color),
    Clear(ClearType),
}

impl fmt::Display for TestBackendOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Write(_) => "Write(_)",
            Flush => "Flush",
            EnableRawMode => "EnableRawMode",
            DisableRawMode => "DisableRawMode",
            HideCursor => "HideCursor",
            ShowCursor => "ShowCursor",
            GetCursor(_, _) => "GetCursor(_, _)",
            SetCursor(_, _) => "SetCursor(_, _)",
            MoveCursor(_) => "MoveCursor(_)",
            Scroll(_) => "Scroll(_)",
            SetAttributes(_) => "SetAttributes(_)",
            RemoveAttributes(_) => "RemoveAttributes(_)",
            SetFg(_) => "SetFg(_)",
            SetBg(_) => "SetBg(_)",
            Clear(_) => "Clear(_)",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
struct TestBackendOpIter<I>(std::iter::Enumerate<I>);

fn err<G: fmt::Display>(op_i: usize, expected: TestBackendOp, got: G) -> ! {
    panic!("op {}: expected {:?}, got {}", op_i, expected, got);
}

impl<I: Iterator<Item = TestBackendOp>> TestBackendOpIter<I> {
    fn next_matches(&mut self, op: TestBackendOp) {
        use std::mem::discriminant;
        let (op_i, expected_op) = self.0.next().unwrap();
        if discriminant(&expected_op) != discriminant(&op) {
            err(op_i, expected_op, op);
        }
    }

    fn next_write(&mut self) -> Vec<u8> {
        match self.0.next().unwrap() {
            (_, Write(w)) => w,
            (i, expected) => err(i, expected, "Write(_)"),
        }
    }

    fn next_flush(&mut self) {
        self.next_matches(Flush);
    }

    fn next_enable_raw_mode(&mut self) {
        self.next_matches(EnableRawMode);
    }

    fn next_disable_raw_mode(&mut self) {
        self.next_matches(DisableRawMode);
    }

    fn next_hide_cursor(&mut self) {
        self.next_matches(HideCursor);
    }

    fn next_show_cursor(&mut self) {
        self.next_matches(ShowCursor);
    }

    fn next_get_cursor(&mut self) -> (u16, u16) {
        match self.0.next().unwrap() {
            (_, GetCursor(x, y)) => (x, y),
            (i, expected) => err(i, expected, "GetCursor(_, _)"),
        }
    }

    fn next_set_cursor(&mut self) -> (u16, u16) {
        match self.0.next().unwrap() {
            (_, SetCursor(x, y)) => (x, y),
            (i, expected) => err(i, expected, "SetCursor(_, _)"),
        }
    }

    fn next_move_cursor(&mut self) -> MoveDirection {
        match self.0.next().unwrap() {
            (_, MoveCursor(m)) => m,
            (i, expected) => err(i, expected, "MoveCursor(_)"),
        }
    }

    fn next_scroll(&mut self) -> i16 {
        match self.0.next().unwrap() {
            (_, Scroll(s)) => s,
            (i, expected) => err(i, expected, "Scroll(_)"),
        }
    }

    fn next_set_attributes(&mut self) -> Attributes {
        match self.0.next().unwrap() {
            (_, SetAttributes(a)) => a,
            (i, expected) => err(i, expected, "SetAttributes(_)"),
        }
    }

    fn next_remove_attributes(&mut self) -> Attributes {
        match self.0.next().unwrap() {
            (_, RemoveAttributes(a)) => a,
            (i, expected) => err(i, expected, "RemoveAttributes(_)"),
        }
    }

    fn next_set_fg(&mut self) -> Color {
        match self.0.next().unwrap() {
            (_, SetFg(c)) => c,
            (i, expected) => err(i, expected, "SetFg(_)"),
        }
    }

    fn next_set_bg(&mut self) -> Color {
        match self.0.next().unwrap() {
            (_, SetBg(c)) => c,
            (i, expected) => err(i, expected, "SetBg(_)"),
        }
    }

    fn next_clear(&mut self) -> ClearType {
        match self.0.next().unwrap() {
            (_, Clear(c)) => c,
            (i, expected) => err(i, expected, "Clear(_)"),
        }
    }
}

#[derive(Debug)]
pub struct TestBackend<I> {
    expected_ops: TestBackendOpIter<I>,
    size: BSize,
}

impl<I: Iterator<Item = TestBackendOp>> TestBackend<I> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<IntoIter: IntoIterator<IntoIter = I, Item = TestBackendOp>>(
        iter: IntoIter,
        size: BSize,
    ) -> Self {
        Self {
            expected_ops: TestBackendOpIter(iter.into_iter().enumerate()),
            size,
        }
    }
}

impl<I: Iterator<Item = TestBackendOp>> Write for TestBackend<I> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        assert_eq!(
            buf,
            &self.expected_ops.next_write()[..],
            "FAILED {:?}",
            std::str::from_utf8(buf)
        );
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.expected_ops.next_flush();
        Ok(())
    }
}

impl<I: Iterator<Item = TestBackendOp>> Backend for TestBackend<I> {
    fn enable_raw_mode(&mut self) -> error::Result<()> {
        self.expected_ops.next_enable_raw_mode();
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> error::Result<()> {
        self.expected_ops.next_disable_raw_mode();
        Ok(())
    }

    fn hide_cursor(&mut self) -> error::Result<()> {
        self.expected_ops.next_hide_cursor();
        Ok(())
    }

    fn show_cursor(&mut self) -> error::Result<()> {
        self.expected_ops.next_show_cursor();
        Ok(())
    }

    fn get_cursor(&mut self) -> error::Result<(u16, u16)> {
        Ok(self.expected_ops.next_get_cursor())
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_set_cursor(), (x, y));
        Ok(())
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_move_cursor(), direction);
        Ok(())
    }

    fn scroll(&mut self, dist: i16) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_scroll(), dist);
        Ok(())
    }

    fn set_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_set_attributes(), attributes);
        Ok(())
    }

    fn remove_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_remove_attributes(), attributes);
        Ok(())
    }

    fn set_fg(&mut self, color: Color) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_set_fg(), color);
        Ok(())
    }

    fn set_bg(&mut self, color: Color) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_set_bg(), color);
        Ok(())
    }

    fn clear(&mut self, clear_type: ClearType) -> error::Result<()> {
        assert_eq!(self.expected_ops.next_clear(), clear_type);
        Ok(())
    }

    fn size(&self) -> error::Result<BSize> {
        Ok(self.size)
    }
}

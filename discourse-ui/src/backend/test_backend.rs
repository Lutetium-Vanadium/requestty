use std::io::Write;

use super::{Attributes, Backend, ClearType, Color, MoveDirection, Size as BSize};
use crate::error;
use TestBackendOp::*;

macro_rules! try_panic {
    ($($tt:tt)*) => {
        if !std::thread::panicking() {
            panic!($($tt)*)
        }
    };
}

// Used to allow derive Debug on structs which exist only to print errors, and show dashes for every
// field
struct Dash;
impl std::fmt::Debug for Dash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("_")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestBackendOp {
    Write(Vec<u8>),
    Flush,
    EnableRawMode,
    DisableRawMode,
    HideCursor,
    ShowCursor,
    /// The value to return
    GetCursorPos(u16, u16),
    MoveCursorTo(u16, u16),
    MoveCursor(MoveDirection),
    Scroll(i16),
    SetAttributes(Attributes),
    RemoveAttributes(Attributes),
    SetFg(Color),
    SetBg(Color),
    Clear(ClearType),
}

#[derive(Debug)]
struct TestBackendOpIter<I>(std::iter::Enumerate<I>);

fn err<G: std::fmt::Debug>(op_i: usize, expected: TestBackendOp, got: G) {
    try_panic!("op {}: expected {:?}, got {:?}", op_i, expected, got);
}

impl<I: Iterator<Item = TestBackendOp>> TestBackendOpIter<I> {
    fn next_matches(&mut self, op: TestBackendOp) {
        let (op_i, expected_op) = self.0.next().unwrap();
        if expected_op != op {
            err(op_i, expected_op, op);
        }
    }

    fn next_write(&mut self, buf: &[u8]) {
        let (op_i, expected) = match self.0.next().unwrap() {
            (i, Write(w)) => (i, w),
            (i, expected) => {
                #[derive(Debug)]
                struct Write<'a>(&'a [u8]);

                return err(i, expected, Write(buf));
            }
        };

        if *buf != expected[..] {
            match (std::str::from_utf8(&expected), std::str::from_utf8(buf)) {
                (Ok(expected), Ok(buf)) => {
                    try_panic!(
                        "{}: expected Write({:?}) got Write({:?})",
                        op_i,
                        expected,
                        buf
                    )
                }
                _ => try_panic!(
                    "{}: expected Write({:?}) got Write({:?})",
                    op_i,
                    expected,
                    buf
                ),
            }
        }
    }

    fn next_get_cursor_pos(&mut self) -> (u16, u16) {
        match self.0.next().unwrap() {
            (_, GetCursorPos(x, y)) => (x, y),
            (i, expected) => {
                #[derive(Debug)]
                struct GetCursorPos(Dash, Dash);
                err(i, expected, GetCursorPos(Dash, Dash));
                // err should exit. The only reason it won't is if the thread is unwinding, in which
                // case the value is never used
                (0, 0)
            }
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
        self.expected_ops.next_write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.expected_ops.next_matches(Flush);
        Ok(())
    }
}

impl<I: Iterator<Item = TestBackendOp>> Backend for TestBackend<I> {
    fn enable_raw_mode(&mut self) -> error::Result<()> {
        self.expected_ops.next_matches(EnableRawMode);
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> error::Result<()> {
        self.expected_ops.next_matches(DisableRawMode);
        Ok(())
    }

    fn hide_cursor(&mut self) -> error::Result<()> {
        self.expected_ops.next_matches(HideCursor);
        Ok(())
    }

    fn show_cursor(&mut self) -> error::Result<()> {
        self.expected_ops.next_matches(ShowCursor);
        Ok(())
    }

    fn get_cursor_pos(&mut self) -> error::Result<(u16, u16)> {
        Ok(self.expected_ops.next_get_cursor_pos())
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> error::Result<()> {
        self.expected_ops.next_matches(MoveCursorTo(x, y));
        Ok(())
    }

    fn move_cursor(&mut self, direction: MoveDirection) -> error::Result<()> {
        self.expected_ops.next_matches(MoveCursor(direction));
        Ok(())
    }

    fn scroll(&mut self, dist: i16) -> error::Result<()> {
        self.expected_ops.next_matches(Scroll(dist));
        Ok(())
    }

    fn set_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        self.expected_ops.next_matches(SetAttributes(attributes));
        Ok(())
    }

    fn remove_attributes(&mut self, attributes: Attributes) -> error::Result<()> {
        self.expected_ops.next_matches(RemoveAttributes(attributes));
        Ok(())
    }

    fn set_fg(&mut self, color: Color) -> error::Result<()> {
        self.expected_ops.next_matches(SetFg(color));
        Ok(())
    }

    fn set_bg(&mut self, color: Color) -> error::Result<()> {
        self.expected_ops.next_matches(SetBg(color));
        Ok(())
    }

    fn clear(&mut self, clear_type: ClearType) -> error::Result<()> {
        self.expected_ops.next_matches(Clear(clear_type));
        Ok(())
    }

    fn size(&self) -> error::Result<BSize> {
        Ok(self.size)
    }
}

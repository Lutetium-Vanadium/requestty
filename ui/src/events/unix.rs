#[cfg(feature = "async-std")]
use async_std_dep::{fs::File, io::Read};
#[cfg(feature = "smol")]
use smol_dep::{fs::File, io::AsyncRead};
#[cfg(feature = "tokio")]
use tokio_dep::{fs::File, io::AsyncRead};

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use anes::parser::{KeyCode as AKeyCode, KeyModifiers as AKeyModifiers, Parser, Sequence};
use futures::Stream;

use super::{KeyCode, KeyEvent, KeyModifiers};
use crate::error;

pub struct AsyncEvents {
    tty: File,
    parser: Parser,
}

impl AsyncEvents {
    pub async fn new() -> error::Result<AsyncEvents> {
        let tty = File::open("/dev/tty").await?;
        Ok(Self {
            tty,
            parser: Parser::default(),
        })
    }

    #[cfg(feature = "tokio")]
    fn try_get_events(&mut self, cx: &mut Context<'_>) -> std::io::Result<()> {
        #[cfg(nightly)]
        let mut buf = std::mem::MaybeUninit::uninit_array::<1024>();
        #[cfg(nightly)]
        let mut buf = tokio_dep::io::ReadBuf::uninit(&mut buf);

        #[cfg(not(nightly))]
        let mut buf = [0u8; 1024];
        #[cfg(not(nightly))]
        let mut buf = tokio_dep::io::ReadBuf::new(&mut buf);

        let tty = Pin::new(&mut self.tty);

        if tty.poll_read(cx, &mut buf).is_ready() {
            self.parser.advance(buf.filled(), buf.remaining() == 0);
        }

        Ok(())
    }

    #[cfg(not(feature = "tokio"))]
    fn try_get_events(&mut self, cx: &mut Context<'_>) -> std::io::Result<()> {
        let mut buf = [0u8; 1024];

        let tty = Pin::new(&mut self.tty);

        if let Poll::Ready(read) = tty.poll_read(cx, &mut buf) {
            let read = read?;
            self.parser.advance(&buf[..read], read == buf.len());
        }

        Ok(())
    }
}

impl Stream for AsyncEvents {
    type Item = error::Result<KeyEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Err(e) = self.try_get_events(cx) {
            return Poll::Ready(Some(Err(e.into())));
        }

        loop {
            match self.parser.next() {
                Some(Sequence::Key(code, modifiers)) => {
                    break Poll::Ready(Some(Ok(KeyEvent {
                        code: code.into(),
                        modifiers: modifiers.into(),
                    })))
                }
                Some(_) => continue,
                None => break Poll::Pending,
            }
        }
    }
}

impl From<AKeyModifiers> for KeyModifiers {
    fn from(amodifiers: AKeyModifiers) -> Self {
        let mut modifiers = KeyModifiers::empty();

        if amodifiers.contains(AKeyModifiers::SHIFT) {
            modifiers |= KeyModifiers::SHIFT;
        }
        if amodifiers.contains(AKeyModifiers::CONTROL) {
            modifiers |= KeyModifiers::CONTROL;
        }
        if amodifiers.contains(AKeyModifiers::ALT) {
            modifiers |= KeyModifiers::ALT;
        }

        modifiers
    }
}

impl From<AKeyCode> for KeyCode {
    fn from(code: AKeyCode) -> Self {
        match code {
            AKeyCode::Backspace => KeyCode::Backspace,
            AKeyCode::Enter => KeyCode::Enter,
            AKeyCode::Left => KeyCode::Left,
            AKeyCode::Right => KeyCode::Right,
            AKeyCode::Up => KeyCode::Up,
            AKeyCode::Down => KeyCode::Down,
            AKeyCode::Home => KeyCode::Home,
            AKeyCode::End => KeyCode::End,
            AKeyCode::PageUp => KeyCode::PageUp,
            AKeyCode::PageDown => KeyCode::PageDown,
            AKeyCode::Tab => KeyCode::Tab,
            AKeyCode::BackTab => KeyCode::BackTab,
            AKeyCode::Delete => KeyCode::Delete,
            AKeyCode::Insert => KeyCode::Insert,
            AKeyCode::F(f) => KeyCode::F(f),
            AKeyCode::Char(c) => KeyCode::Char(c),
            AKeyCode::Null => KeyCode::Null,
            AKeyCode::Esc => KeyCode::Esc,
        }
    }
}

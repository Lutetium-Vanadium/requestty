use std::{fmt, io};

pub type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    IoError(io::Error),
    FmtError(fmt::Error),
    Utf8Error(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    NotATty,
    Interrupted,
    Eof,
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorKind::IoError(e) => Some(e),
            ErrorKind::FmtError(e) => Some(e),
            ErrorKind::Utf8Error(e) => Some(e),
            ErrorKind::ParseIntError(e) => Some(e),
            ErrorKind::Interrupted => None,
            ErrorKind::Eof => None,
            ErrorKind::NotATty => None,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IoError(e) => write!(fmt, "IoError: {}", e),
            ErrorKind::FmtError(e) => write!(fmt, "FmtError: {}", e),
            ErrorKind::Utf8Error(e) => write!(fmt, "Utf8Error: {}", e),
            ErrorKind::ParseIntError(e) => write!(fmt, "ParseIntError: {}", e),
            ErrorKind::Interrupted => write!(fmt, "CTRL+C"),
            ErrorKind::Eof => write!(fmt, "EOF"),
            ErrorKind::NotATty => write!(fmt, "Not a tty"),
        }
    }
}

macro_rules! impl_from {
    ($from:path, $e:ident => $body:expr) => {
        impl From<$from> for ErrorKind {
            fn from(e: $from) -> Self {
                let $e = e;
                $body
            }
        }
    };
}

impl_from!(io::Error, e => ErrorKind::IoError(e));
impl_from!(fmt::Error, e => ErrorKind::FmtError(e));
impl_from!(std::string::FromUtf8Error, e => ErrorKind::Utf8Error(e));
impl_from!(std::num::ParseIntError, e => ErrorKind::ParseIntError(e));

#[cfg(feature = "crossterm")]
impl_from!(crossterm::ErrorKind, e =>
    match e {
        crossterm::ErrorKind::IoError(e) => Self::from(e),
        crossterm::ErrorKind::FmtError(e) => Self::from(e),
        crossterm::ErrorKind::Utf8Error(e) => Self::from(e),
        crossterm::ErrorKind::ParseIntError(e) => Self::from(e),
        crossterm::ErrorKind::ResizingTerminalFailure(_)
        | crossterm::ErrorKind::SettingTerminalTitleFailure => unreachable!(),
        _ => unreachable!(),
    }
);

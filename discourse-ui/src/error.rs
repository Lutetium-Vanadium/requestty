//! Error definitions for `discourse`

use std::{fmt, io};

/// The `discourse` result type.
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// The errors that can occur in `discourse`.
#[derive(Debug)]
pub enum ErrorKind {
    /// A regular [`std::io::Error`].
    IoError(io::Error),
    /// This occurs when `Ctrl+C` is received in [`Input`](crate::Input).
    Interrupted,
    /// This occurs when `Null` is received in [`Input`](crate::Input).
    Eof,
}

impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorKind::IoError(e) => Some(e),
            ErrorKind::Interrupted => None,
            ErrorKind::Eof => None,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::IoError(e) => write!(fmt, "IoError: {}", e),
            ErrorKind::Interrupted => write!(fmt, "CTRL+C"),
            ErrorKind::Eof => write!(fmt, "EOF"),
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

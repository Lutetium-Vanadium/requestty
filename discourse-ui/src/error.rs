use std::{fmt, io};

pub type Result<T> = std::result::Result<T, ErrorKind>;

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    IoError(io::Error),
    Interrupted,
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

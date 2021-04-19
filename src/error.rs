use std::{fmt, io};

pub type Result<T> = std::result::Result<T, InquirerError>;

#[derive(Debug)]
#[non_exhaustive]
pub enum InquirerError {
    IoError(io::Error),
    FmtError(fmt::Error),
    Utf8Error(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
    NotATty,
}

impl std::error::Error for InquirerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InquirerError::IoError(e) => Some(e),
            InquirerError::FmtError(e) => Some(e),
            InquirerError::Utf8Error(e) => Some(e),
            InquirerError::ParseIntError(e) => Some(e),
            InquirerError::NotATty => None,
        }
    }
}

impl fmt::Display for InquirerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InquirerError::IoError(e) => write!(fmt, "IoError: {}", e),
            InquirerError::FmtError(e) => write!(fmt, "FmtError: {}", e),
            InquirerError::Utf8Error(e) => write!(fmt, "Utf8Error: {}", e),
            InquirerError::ParseIntError(e) => write!(fmt, "ParseIntError: {}", e),
            InquirerError::NotATty => write!(fmt, "Not a tty"),
        }
    }
}

macro_rules! impl_from {
    ($from:path, $e:ident => $body:expr) => {
        impl From<$from> for InquirerError {
            fn from(e: $from) -> Self {
                let $e = e;
                $body
            }
        }
    };
}

impl_from!(io::Error, e => InquirerError::IoError(e));
impl_from!(fmt::Error, e => InquirerError::FmtError(e));
impl_from!(std::string::FromUtf8Error, e => InquirerError::Utf8Error(e));
impl_from!(std::num::ParseIntError, e => InquirerError::ParseIntError(e));
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

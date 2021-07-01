#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    Abort,
    IoError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Abort => f.write_str("aborted"),
            Self::IoError => f.write_str("I/O error"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}

pub type Result<T> = std::result::Result<T, Error>;

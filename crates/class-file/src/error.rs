use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Error {
    path: PathBuf,
    message: String,
}

impl Error {
    pub(crate) fn new(path: &Path, message: impl fmt::Display) -> Self {
        Self {
            path: path.to_path_buf(),
            message: message.to_string(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.message)
    }
}

impl std::error::Error for Error {}

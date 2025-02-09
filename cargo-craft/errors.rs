use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Error {
    ShellCommandError(String),
    IOError(String),
}

impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Error", 2)?;
        s.serialize_field("variant", &self.variant())?;
        s.serialize_field("message", &format!("{}", self))?;
        s.end()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.variant(),
            match self {
                Self::ShellCommandError(s) => format!("{}", s),
                Self::IOError(s) => format!("{}", s),
            }
        )
    }
}

impl Error {
    pub fn variant(&self) -> String {
        match self {
            Error::ShellCommandError(_) => "ShellCommandError",
            Error::IOError(_) => "IOError",
        }
        .to_string()
    }
}

impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(format!("{}", e))
    }
}
impl From<iocore::Exception> for Error {
    fn from(e: iocore::Exception) -> Self {
        Error::IOError(format!("{}", e))
    }
}
pub type Result<T> = std::result::Result<T, Error>;

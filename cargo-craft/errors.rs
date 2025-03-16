use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Error {
    ShellCommandError(String),
    IOError(String),
    SerializationError(String),
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
            "{}: {}",
            self.variant(),
            match self {
                Self::ShellCommandError(s) => format!("{}", s),
                Self::IOError(s) => format!("{}", s),
                Self::SerializationError(s) => format!("{}", s),
            }
        )
    }
}

impl Error {
    pub fn variant(&self) -> String {
        match self {
            Error::ShellCommandError(_) => "ShellCommandError",
            Error::IOError(_) => "IOError",
            Error::SerializationError(_) => "SerializationError",
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
impl From<iocore::Error> for Error {
    fn from(e: iocore::Error) -> Self {
        Error::IOError(format!("{}", e))
    }
}
// impl From<toml::ser::Error> for Error {
//     fn from(e: toml::ser::Error) -> Self {
//         Error::SerializationError(format!("{}", e))
//     }
// }
pub type Result<T> = std::result::Result<T, Error>;

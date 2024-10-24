use serde::{Serializer, Serialize};
use serde::ser::SerializeStruct;
use std::fmt::Display;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidUtf8(String),
    ParseIntError(String),
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
                Self::InvalidUtf8(s) => format!("{}", s),
                Self::ParseIntError(s) => format!("{}", s),
            }
        )
    }
}

impl Error {
    pub fn variant(&self) -> String {
        match self {
            Error::InvalidUtf8(_) => "InvalidUtf8",
            Error::ParseIntError(_) => "ParseIntError",
        }
        .to_string()
    }
}

impl std::error::Error for Error {}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::ParseIntError(format!("{}", e))
    }
}
impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error::InvalidUtf8(format!("{}", e))
    }
}
pub type Result<T> = std::result::Result<T, Error>;

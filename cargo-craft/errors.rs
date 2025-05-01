use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt::Display;
use std::error::Error as StdErr;

#[derive(Debug, Clone)]
pub enum Error {
    ShellCommandError(String),
    IOError(String),
    SerializationError(String),
    ParseError(String),
    TemplateError(String),
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
                Self::ShellCommandError(e) => e.to_string(),
                Self::IOError(e) => e.to_string(),
                Self::SerializationError(e) => e.to_string(),
                Self::ParseError(e) => e.to_string(),
                Self::TemplateError(e) => e.to_string(),
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
            Error::ParseError(_) => "ParseError",
            Error::TemplateError(_) => "TemplateError",
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
impl From<clap::Error> for Error {
    fn from(e: clap::Error) -> Self {
        Error::ParseError(format!("{}", e))
    }
}
impl From<tera::Error> for Error {
    fn from(e: tera::Error) -> Self {
        Error::TemplateError(format!("{}{}", e, e.source().map(|e|format!(": {}", e)).unwrap_or_default()))
    }
}
// impl From<toml::ser::Error> for Error {
//     fn from(e: toml::ser::Error) -> Self {
//         Error::SerializationError(format!("{}", e))
//     }
// }
pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! traceback {
    ($variant:ident, $error:expr ) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let name = name.strip_suffix("::f").unwrap();
        $crate::Error::$variant(format!("{} [{}:[{}:{}]]\n", $error, name, file!(), line!()))
    }};
    ($variant:ident, $format:literal, $arg:expr  ) => {{
        $crate::traceback!($variant, format!($format, $arg))
    }};
    ($variant:ident, $format:literal, $( $arg:expr ),* ) => {{
        $crate::traceback!($variant, format!($format, $($arg,)*))
    }};
}

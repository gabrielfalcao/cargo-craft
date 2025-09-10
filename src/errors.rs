use crate::ClapExecuter;
use serde::{Deserialize, Serialize};
use std::error::Error as StdErr;
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum Error {
    ShellCommandError(String),
    IOError(String),
    SerializationError(String),
    ParseError(String),
    TemplateError(String),
    JsonError(String),
    RuntimeError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.variant(),
            match self {
                Error::ShellCommandError(e) => e.to_string(),
                Error::IOError(e) => e.to_string(),
                Error::SerializationError(e) => e.to_string(),
                Error::ParseError(e) => e.to_string(),
                Error::TemplateError(e) => e.to_string(),
                Error::JsonError(e) => e.to_string(),
                Error::RuntimeError(e) => e.to_string(),
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
            Error::JsonError(_) => "JsonError",
            Error::RuntimeError(_) => "RuntimeError",
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
        Error::TemplateError(format!(
            "{}{}",
            e,
            e.source().map(|e| format!(": {}", e)).unwrap_or_default()
        ))
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonError(e.to_string())
    }
}
// impl From<toml::ser::Error> for Error {
//     fn from(e: toml::ser::Error) -> Self {
//         Error::SerializationError(format!("{}", e))
//     }
// }

#[derive(Debug, Clone)]
pub enum ExecutionResult<T: ClapExecuter> {
    Ok(T),
    Err(T, Error),
}

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

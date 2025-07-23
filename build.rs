use std::env;
use std::fs;
use std::path::{Path,PathBuf};

const SCRIPT: &'static str = include_str!("./cargo-craft.sh");

fn main() -> Result<(), String> {
    let cargo_env_path = canonicalize!(PathBuf::from(cargo_env_path()));
    let cargo_path = option!(cargo_env_path.parent().map(Path::to_path_buf), result!(cargo_bin_path()));

    let cargo_craft_bin = cargo_path.join("cargo-craft");

    for entry in result!(std::fs::read_dir("cargo-craft")) {
        let entry = result!(entry);
        println!("rerun-if-changed={}", entry.path().display());
    }

    eprintln!("writing to {}", cargo_craft_bin.display());
    result!(fs::write(&cargo_craft_bin, SCRIPT));
    Ok(())
}
fn user_home() -> String {
    env::var("HOME").map(String::from).expect("USER HOME")
}
fn cargo_env_path() -> String {
    env::var("CARGO")
        .map(String::from)
        .unwrap_or_else(|_| format!("{}/.cargo/bin/cargo", user_home()))
}
fn cargo_bin_path() -> Result<PathBuf, String> {
    Ok(canonicalize!(PathBuf::from(String::from(
        "~/.cargo/bin/cargo"
    ))))
}

#[macro_export]
macro_rules! result {
    ($result:expr) => {
        $result.map_err(|err| err.to_string())?
    };
}
#[macro_export]
macro_rules! canonicalize {
    ($path:expr) => {
        result!($path.canonicalize())
    };
}

#[macro_export]
macro_rules! option {
    ($option:expr, $err_message:literal, $( $arg:expr ),* $(,)? ) => {
        match $option{
            Some(value) => value,
            None => return Err(format!($err_message, $($arg,)*))
        }
    };
    ($option:expr, $fallback:expr) => {
        match $option{
            Some(value) => value,
            None => $fallback
        }
    };
}

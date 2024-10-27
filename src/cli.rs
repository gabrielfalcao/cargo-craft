use crate::helpers::extend_table;

use clap::Parser;
use iocore::Path;
use regex::Regex;
use toml::{Table, Value};

pub fn slug(text: &str, sep: Option<&str>) -> String {
    let re = Regex::new("[^a-zA-Z0-9-]+").unwrap();
    re.replace_all(text, sep.unwrap_or("-").to_string())
        .to_string()
}
pub fn acceptable_crate_name(val: &str) -> ::std::result::Result<String, String> {
    let re = Regex::new("[^a-zA-Z0-9_-]+").unwrap();
    if re.is_match(val) {
        Err(format!(
            "{:#?} does not appear to be a valid crate name",
            val
        ))
    } else {
        Ok(val.to_string())
    }
}
pub fn valid_package_name(val: &str) -> ::std::result::Result<String, String> {
    let re = Regex::new("[^a-zA-Z0-9_]+").unwrap();
    if re.is_match(val) {
        Err(format!("{:#?} is not a valid package name", val))
    } else {
        Ok(val.to_string())
    }
}
pub fn path_to_entry_path(entry: Option<Table>) -> Option<Path> {
    match entry?.get("path")? {
        Value::String(path) => Some(Path::new(path)),
        _ => None,
    }
}

pub fn valid_manifest_path(val: &str) -> ::std::result::Result<Path, String> {
    let val = acceptable_crate_name(val)?;
    let path = Path::new(val);
    let manifest_path = path.join("Cargo.toml");
    if manifest_path.is_file() {
        return Err(format!("{} already exists", &manifest_path));
    }
    Ok(path)
}

#[derive(Parser, Debug)]
pub struct Craft {
    #[arg(value_parser = valid_manifest_path)]
    at: Path,

    #[arg(short, long, value_parser = valid_package_name)]
    package_name: Option<String>,

    #[arg(long, default_value = "0.1.0")]
    version: String,

    #[arg(short, long)]
    dep: Vec<String>,

    #[arg(short, long)]
    pub cli: bool,

    #[arg(short, long)]
    bin: Vec<String>,

    #[arg(long)]
    lib_path: Option<String>,

    #[arg(long, default_value = "cli")]
    bin_path: String,
}

impl Craft {
    pub fn crate_name(&self) -> String {
        self.at.name()
    }
    pub fn package_name(&self) -> String {
        slug(
            &self
                .package_name
                .clone()
                .or(Some(self.crate_name()))
                .unwrap(),
            Some("_"),
        )
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn lib_path(&self) -> String {
        self.lib_path.clone().unwrap_or(self.crate_name())
    }

    pub fn lib_options() -> Table {
        let mut options = Table::new();
        for falsy in ["doctest", "bench"] {
            options.insert(falsy.to_string(), Value::Boolean(false));
        }
        options
    }

    pub fn bin_options() -> Table {
        let mut options = Craft::lib_options();
        for falsy in ["doc"] {
            options.insert(falsy.to_string(), Value::Boolean(false));
        }
        options
    }

    pub fn bin_names(&self) -> Vec<String> {
        let mut binaries = self.bin.clone();
        if self.bin.len() == 0 && self.cli {
            binaries.push(self.crate_name());
        }
        binaries
    }

    pub fn bin_entries(&self) -> Vec<Table> {
        let mut entries = Vec::<Table>::new();
        for name in self.bin_names() {
            let mut table = Table::new();
            table.insert("name".to_string(), Value::String(name.clone()));
            table.insert(
                "path".to_string(),
                Value::String(
                    Path::new(&self.bin_path)
                        .join(format!("{}.rs", name))
                        .to_string(),
                ),
            );
            table = extend_table(&Craft::bin_options(), &table);
            entries.push(table);
        }
        entries
    }

    pub fn lib_entry(&self) -> Option<Table> {
        let mut entry = Table::new();
        entry.insert("name".to_string(), Value::String(self.package_name()));
        entry.insert(
            "path".to_string(),
            Value::String(Path::new(&self.lib_path()).join("lib.rs").to_string()),
        );
        Some(extend_table(&Craft::bin_options(), &entry))
    }

    pub fn errors_entry(&self) -> Option<Table> {
        let mut entry = Table::new();
        entry.insert(
            "path".to_string(),
            Value::String(Path::new(&self.lib_path()).join("errors.rs").to_string()),
        );
        Some(extend_table(&Craft::bin_options(), &entry))
    }

    pub fn path(&self) -> Path {
        self.at.clone()
    }
    pub fn path_to(&self, to: impl Into<String>) -> Path {
        self.path().join(to)
    }

    pub fn manifest_path(&self) -> Path {
        self.path_to("Cargo.toml")
    }
}

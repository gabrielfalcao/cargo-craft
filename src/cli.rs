use crate::helpers::extend_table;

use clap::Parser;
use iocore::Path;
use regex::Regex;
use toml::{Table, Value};

pub fn slug(text: &str, sep: Option<&str>) -> String {
    let re = Regex::new("[^a-zA-Z0-9_-]").unwrap();
    re.replace_all(text, sep.unwrap_or("-").to_string())
        .to_string()
}
pub fn acceptable_crate_name(val: &str) -> ::std::result::Result<String, String> {
    let re = Regex::new(r"^[a-z]+([-][a-z0-9]+|[a-z0-9]+)+$").unwrap();
    if re.is_match(val) {
        Ok(val.to_string())
    } else {
        Err(format!(
            "{:#?} does not appear to be a valid crate name",
            val
        ))
    }
}
pub fn valid_package_name(val: &str) -> ::std::result::Result<String, String> {
    let re = Regex::new(r"^[a-z]+([_][a-z0-9]+|[a-z0-9]+)+$").unwrap();
    if re.is_match(val) {
        Ok(val.to_string())
    } else {
        Err(format!("{:#?} is not a valid package name", val))
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
    let path = if path.name() == "Cargo.toml" && !path.is_dir() {
        path.parent().expect(&format!("parent of {}", &path))
    } else {
        path.clone()
    };
    let manifest_path = path.join("Cargo.toml");
    if manifest_path.try_canonicalize().exists() && !manifest_path.is_dir() {
        return Err(format!("{} already exists", &manifest_path));
    }
    Ok(path)
}

pub fn crate_name_from_path(path: impl Into<Path>) -> ::std::result::Result<String, String> {
    let name = path.into().without_extension().name();
    let crate_name = into_acceptable_crate_name(&name);
    Ok(crate_name)
}
pub fn package_name_from_string_or_path(name: Option<String>, path: impl Into<Path>) -> ::std::result::Result<String, String> {
    let name = match name {
        Some(name) => name,
        None => crate_name_from_path(path)?
    };
    let package_name = into_acceptable_package_name(&name);
    Ok(package_name)
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

    #[arg(short, long)]
    pub value_enum: bool,
}

impl Craft {
    pub fn crate_name(&self) -> String {
        crate_name_from_path(&self.at).unwrap()
    }
    pub fn package_name(&self) -> String {
        package_name_from_string_or_path(self.package_name.clone(), &self.at).unwrap()
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn lib_path(&self) -> Path {
        crate_name_from_path(&self.lib_path.clone().unwrap_or_else(||self.crate_name())).unwrap().into()
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

    pub fn git_entries(&self) -> Vec<Table> {
        let mut entries = Vec::<Table>::new();
        for name in vec![".gitignore"] {
            let mut table = Table::new();
            table.insert("name".to_string(), Value::String(name.to_string()));
            table.insert(
                "path".to_string(),
                Value::String(self.path().join(name).to_string()),
            );
            entries.push(table);
        }
        entries
    }

    pub fn lib_entry(&self) -> Option<Table> {
        let mut entry = Table::new();
        entry.insert("name".to_string(), Value::String(self.package_name()));
        entry.insert(
            "path".to_string(),
            Value::String(self.lib_path().join("lib.rs").to_string()),
        );
        Some(extend_table(&Craft::bin_options(), &entry))
    }

    pub fn errors_entry(&self) -> Option<Table> {
        let mut entry = Table::new();
        entry.insert(
            "path".to_string(),
            Value::String(self.lib_path().join("errors.rs").to_string()),
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
    pub fn deps(&self) -> Vec<String> {
        self.dep.to_vec()
    }
}

pub fn into_acceptable_crate_name(val: &str) -> String {
    into_acceptable_name(val, '-')
}

pub fn into_acceptable_package_name(val: &str) -> String {
    into_acceptable_name(val, '_')
}
pub fn into_acceptable_name(val: &str, sep: char) -> String {
    let val = val.to_lowercase();
    let re = Regex::new(r"^[^a-z]+").unwrap();
    let val = re.replace_all(&val, String::new()).to_string();
    let re = Regex::new(r"[^a-z0-9]$").unwrap();
    let val = re.replace_all(&val, String::new()).to_string();
    let re = Regex::new(&format!(r"[{}]+", sep)).unwrap();
    let val = re.replace_all(&val, String::from(sep)).to_string();
    let re = Regex::new(&format!(r"[^a-zA-Z0-9{}]", sep)).unwrap();
    let val = re.replace_all(&val, String::from(sep)).to_string();
    val
}

use crate::cargo_manifest::CargoManifest;
use crate::helpers::extend_table;

use clap::Parser;
use iocore::Path;
use toml::{Table, Value};

pub fn valid_manifest_path(val: &str) -> ::std::result::Result<Path, String> {
    let path = Path::new(val);
    let manifest_path = path.join("Cargo.toml");
    if manifest_path.is_file() {
        return Err(format!("{} already exists", &manifest_path));
    }
    Ok(path)
}

pub fn path_to_entry_path(entry: Option<Table>) -> Option<Path> {
    match entry?.get("path")? {
        Value::String(path) => Some(Path::new(path)),
        _ => None,
    }
}

#[derive(Parser, Debug)]
pub struct Craft {
    #[arg()]
    name: String,

    #[arg(long, default_value = "0.1.0")]
    version: String,

    #[arg(short, long, default_value = ".", value_parser = valid_manifest_path)]
    at: Path,

    #[arg(short, long)]
    dep: Vec<String>,

    #[arg(short, long)]
    pub lib: bool,

    #[arg(short, long, required_unless_present("lib"))]
    pub cli: bool,

    #[arg(short, long)]
    bin: Vec<String>,

    #[arg(long)]
    lib_path: Option<String>,

    #[arg(long, default_value = "cli")]
    bin_path: String,
}

impl Craft {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn lib_path(&self) -> String {
        self.lib_path.clone().unwrap_or(self.name.clone())
    }

    pub fn lib_options() -> Table {
        let mut options = Table::new();
        for falsy in ["doctest", "bench"] {
            options
                .insert(falsy.to_string(), Value::Boolean(false));
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
            binaries.push(self.name());
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
        if self.lib {
            let mut entry = Table::new();
            entry.insert("name".to_string(), Value::String(self.name.to_string()));
            entry.insert(
                "path".to_string(),
                Value::String(
                    Path::new(&self.lib_path())
                        .join(format!("{}.rs", self.name()))
                        .to_string(),
                ),
            );
            Some(extend_table(&Craft::bin_options(), &entry))
        } else {
            None
        }
    }

    pub fn errors_entry(&self) -> Option<Table> {
        if self.lib {
            let mut entry = Table::new();
            entry.insert("name".to_string(), Value::String(self.name.to_string()));
            entry.insert(
                "path".to_string(),
                Value::String(
                    Path::new(&self.bin_path)
                        .join("errors.rs")
                        .to_string(),
                ),
            );
            Some(extend_table(&Craft::bin_options(), &entry))
        } else {
            None
        }
    }

    pub fn cargo_manifest(&self) -> CargoManifest {
        let mut manifest = CargoManifest::default();
        manifest.set_package_key_value("name", self.name());
        manifest.set_package_key_value("version", self.version());
        manifest.set_lib_entry(self.lib_entry());
        manifest.set_bin_entries(self.bin_entries());
        manifest
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

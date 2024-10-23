use cargo_craft::extend_table;
use cargo_craft::CargoManifest;
use clap::Parser;
use iocore::Path;
use toml::{Table, Value};

pub fn valid_manifest_path(val: &str) -> ::std::result::Result<Path, String> {
    let path = Path::safe(val).map_err(|e| e.to_string())?;
    let manifest_path = path.join("Cargo.toml").try_canonicalize();
    if manifest_path.is_file() {
        return Err(format!("{} already exists", &manifest_path));
    }
    Ok(path)
}

#[derive(Parser, Debug)]
pub struct Craft {
    #[arg()]
    pub name: String,

    #[arg(short, long, default_value = ".", value_parser = valid_manifest_path)]
    at: Path,

    #[arg(short, long)]
    dep: Vec<String>,

    #[arg(short, long)]
    lib: bool,

    #[arg(short, long)]
    cli: bool,

    #[arg(short, long)]
    force: bool,

    #[arg(short, long)]
    bin: Vec<String>,

    #[arg(long)]
    lib_path: Option<String>,

    #[arg(long, default_value = "cli")]
    bin_path: String,
}
impl Craft {
    pub fn lib_options() -> Table {
        let mut options = Table::new();
        for falsy in ["doctest", "bench"] {
            options
                .insert(falsy.to_string(), Value::Boolean(false))
                .unwrap();
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
            binaries.push(self.name.clone());
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
                    Path::new(&self.bin_path)
                        .join(format!("{}.rs", &self.name))
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
        manifest.set_package_key_value("name", self.name.clone());
        manifest.set_lib_entry(self.lib_entry());
        manifest.set_bin_entries(self.bin_entries());
        manifest
    }
}

fn main() {
    let args = Craft::parse();
    let manifest = args.cargo_manifest();
    println!("{}", manifest.to_string_pretty().unwrap());
}

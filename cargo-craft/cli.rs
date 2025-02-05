use crate::helpers::*;
use crate::shell::*;
use crate::templates::{render, render_cli};
use clap::Parser;
use iocore::Path;
use toml::{Table, Value};

#[derive(Parser, Debug)]
pub struct Craft {
    #[arg(value_parser = valid_manifest_path)]
    at: Path,

    #[arg(short, long, value_parser = valid_package_name)]
    package_name: Option<String>,

    #[arg(long, default_value = "0.0.1")]
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
pub trait ClapExecuter: Parser {
    fn run(args: &Self) -> Result<(), String>;
    fn main() -> Result<(), String> {
        let args = Self::parse_from(Self::args());
        Self::run(&args)?;
        Ok(())
    }
    fn args() -> Vec<String> {
        let args = std::env::args()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>();
        if args[0] == "cargo" {
            dbg!(args[0..].to_vec())
        } else {
            dbg!(args)
        }
    }
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
        crate_name_from_path(&self.lib_path.clone().unwrap_or_else(|| self.crate_name()))
            .unwrap()
            .into()
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

impl ClapExecuter for Craft {
    fn run(args: &Craft) -> Result<(), String> {
        let manifest_path = args.manifest_path();
        let manifest_string = render(&args, "Cargo.toml").unwrap();
        manifest_path.write(&manifest_string.as_bytes()).unwrap();
        println!("wrote {}", manifest_path);

        let mut ttargets = vec![
            (render(&args, "lib.rs"), vec![args.lib_entry()]),
            (render(&args, "errors.rs"), vec![args.errors_entry()]),
            (
                render_cli(&args),
                args.bin_entries()
                    .iter()
                    .map(|entry| Some(entry.clone()))
                    .collect::<Vec<Option<Table>>>(),
            ),
        ];
        ttargets.extend(args.git_entries().iter().map(|entry| {
            let name = entry
                .get("name")
                .expect("entry name")
                .as_str()
                .expect("str");
            (render(&args, name), vec![Some(entry.clone())])
        }));
        for (template, target) in ttargets {
            for target in target
                .iter()
                .filter(|entry| entry.is_some())
                .map(|entry| path_to_entry_path(entry.clone()))
                .filter(|path| path.is_some())
                .map(|path| path.unwrap())
                .collect::<Vec<Path>>()
            {
                match args
                    .path_to(target)
                    .write(&template.clone().unwrap().as_bytes())
                {
                    Ok(path) => {
                        println!("wrote {}", path);
                    }
                    Err(err) => panic!("{}", err),
                }
            }
        }
        cargo_add("serde -F derive", args.path());
        if args.cli || args.bin_entries().len() > 0 {
            cargo_add("clap -F derive,env,string,unicode,wrap_help", args.path());
        }
        for dep in args.deps() {
            cargo_add(&dep, args.path());
        }
        Ok(())
    }
}
fn cargo_add(dep: impl std::fmt::Display, current_dir: Path) {
    shell_command(format!("cargo-unix add {}", dep), current_dir);
}

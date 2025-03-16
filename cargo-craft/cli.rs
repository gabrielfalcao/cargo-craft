use crate::errors::Result;
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

    #[arg(long, default_value = ".")]
    bin_path: String,

    #[arg(short = 'V', long)]
    pub value_enum: bool,

    #[arg(short, long)]
    pub verbose: bool,
}
pub trait ClapExecuter: Parser {
    fn run(args: &Self) -> Result<()>;
    fn main() -> Result<()> {
        let args = Self::parse_from(Self::args());
        Self::run(&args)?;
        Ok(())
    }
    fn args() -> Vec<String> {
        let args = std::env::args()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>();
        let execname = Path::new(&args[0]).name();
        let args = if execname.ends_with("cargo") || execname.ends_with("cargo-craft") {
            args[1..].to_vec()
        } else {
            args.to_vec()
        };
        args
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
        for name in vec![".gitignore", ".rustfmt.toml"] {
            let mut table = Table::new();
            table.insert("name".to_string(), Value::String(name.to_string()));
            table.insert(
                "path".to_string(),
                Value::String(
                    Path::new(&self.bin_path)
                        .join(&name)
                        .to_string(),
                ),
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
    pub fn path_to(&self, to: impl std::fmt::Display) -> Path {
        self.path().join(to.to_string())
    }

    pub fn manifest_path(&self) -> Path {
        self.path_to("Cargo.toml")
    }
    pub fn deps(&self) -> Vec<String> {
        self.dep.to_vec()
    }
    pub fn go(&self) -> Result<()> {
        let manifest_path = self.manifest_path();
        let manifest_string = render(&self, "Cargo.toml").unwrap();
        manifest_path.write(&manifest_string.as_bytes()).unwrap();
        println!("wrote {}", manifest_path);

        let mut ttargets = vec![
            (render(&self, "lib.rs"), vec![self.lib_entry()]),
            (render(&self, "errors.rs"), vec![self.errors_entry()]),
            (
                render_cli(&self),
                self.bin_entries()
                    .iter()
                    .map(|entry| Some(entry.clone()))
                    .collect::<Vec<Option<Table>>>(),
            ),
        ];
        let git_entries = self.git_entries().into_iter().map(|entry| {
            let name = entry
                .get("name")
                .expect("entry name")
                .as_str()
                .expect("str");
            (render(&self, name), vec![Some(entry.clone())])
        });
        ttargets.extend(git_entries);
        let ttargets = ttargets
            .iter()
            .filter(|(path, _)| path.is_some())
            .map(|(path, entries)| (path.clone().unwrap(), entries.clone()))
            .collect::<Vec<(String, Vec<Option<Table>>)>>();
        let mut written_paths = Vec::<Path>::new();
        for (template, target) in ttargets {
            for target in target
                .iter()
                .filter(|entry| entry.is_some())
                .map(|entry| path_to_entry_path(entry.clone()))
                .filter(|path| path.is_some())
                .map(|path| path.unwrap())
                .collect::<Vec<Path>>()
            {
                match self
                    .path_to(target)
                    .write(&template.as_bytes())
                {
                    Ok(path) => {
                        println!("wrote {}", path);
                        written_paths.push(path);
                    }
                    Err(err) => panic!("{}", err),
                }
            }
        }
        for target in written_paths {
            if target.extension().unwrap_or_default().ends_with("rs") {
                shell_command(
                    format!("rustfmt {}", target.relative_to_cwd()),
                    Path::cwd(),
                    self.verbose,
                )?;
            }
        }
        // shell_command(format!("tput clear"), self.path(), self.verbose)?;
        cargo_add("serde -F derive", self.path(), self.verbose)?;
        cargo_add("iocore", self.path(), self.verbose)?;
        if self.cli || self.bin_entries().len() > 0 {
            cargo_add(
                "clap -F derive,env,string,unicode,wrap_help",
                self.path(),
                self.verbose,
            )?;
        }
        for dep in self.deps() {
            cargo_add(&dep, self.path(), self.verbose)?;
        }
        Ok(())
    }
}

impl ClapExecuter for Craft {
    fn run(args: &Craft) -> Result<()> {
        let could_rollback = !args.path().try_canonicalize().exists();
        match args.go() {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("error {}", e);
                if could_rollback {
                    eprintln!("rolling back {}", args.path());
                    args.path().delete()?;
                }
                Ok(())
            }
        }
    }
}

pub fn cargo_add(dep: impl std::fmt::Display, current_dir: Path, verbose: bool) -> Result<()> {
    shell_command(format!("cargo-unix add {}", dep), current_dir, verbose)?;
    Ok(())
}

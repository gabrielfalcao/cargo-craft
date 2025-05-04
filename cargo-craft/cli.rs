use crate::errors::Result;
use crate::helpers::*;
use crate::templates::{render, render_cli};
use crate::traceback;
use clap::Parser;
use iocore::Path;
use toml::{Table, Value};
use clap::CommandFactory;

#[derive(Parser, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Craft {
    #[arg()]
    pub at: Path,

    #[arg(short, long, value_parser = valid_package_name)]
    pub package_name: Option<String>,

    #[arg(long, default_value = "0.0.1")]
    pub version: String,

    #[arg(short, long)]
    pub dep: Vec<String>,

    #[arg(short, long)]
    pub cli: bool,

    #[arg(short, long)]
    pub bin: Vec<String>,

    #[arg(long)]
    pub lib_path: Option<String>,

    #[arg(long, default_value = ".")]
    pub bin_path: String,

    #[arg(short = 'V', long)]
    pub value_enum: bool,

    #[arg(short, long)]
    pub subcommands: bool,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long, help = "cargo add --quiet")]
    pub quiet_add: bool,

    #[arg(short, long)]
    pub offline: bool,

    #[arg(short = 'e', long)]
    pub add_error_type: Vec<String>,

    #[arg(short, long)]
    pub no_rollback: bool,

    #[arg(short, long)]
    pub force: bool,
}
pub trait ClapExecuter: Parser + std::fmt::Debug {
    fn run(args: &Self) -> Result<()>;
    fn main() -> Result<()> {
        let args = Self::parse_from(Self::args());
        Self::run(&args)?;
        Ok(())
    }
    fn args() -> Vec<String> {
        let args = iocore::env::args();
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

    pub fn struct_name(&self) -> String {
        struct_name_from_package_name(&self.package_name())
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
        if !binaries.contains(&self.crate_name()) {
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
        for name in vec![
            ".gitignore",
            ".rustfmt.toml",
            "rust-toolchain.toml",
            "README.md",
        ] {
            let mut table = Table::new();
            table.insert("name".to_string(), Value::String(name.to_string()));
            table.insert(
                "path".to_string(),
                Value::String(Path::new(&self.bin_path).join(&name).to_string()),
            );
            entries.push(table);
        }
        entries
    }

    pub fn lib_entry(&self, path: impl std::fmt::Display) -> Option<Table> {
        let mut entry = Table::new();
        entry.insert("name".to_string(), Value::String(self.package_name()));
        entry.insert(
            "path".to_string(),
            Value::String(self.lib_path().join(path).to_string()),
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
    pub fn deps(&self) -> Result<Vec<Dependency>> {
        let mut deps = Vec::new();
        for h in self.dep.iter() {
            let mut args = vec![format!("dependency")];
            args.extend(
                h.split(" ")
                    .filter(|h| h.trim().len() > 0)
                    .map(|h| h.to_string()),
            );
            deps.push(Dependency::try_parse_from(args).map_err(|e| traceback!(ParseError, e))?);
        }
        Ok(deps)
    }
    pub fn error_types(&self) -> Vec<String> {
        self.add_error_type
            .iter()
            .map(|h| into_acceptable_error_type_name(h))
            .collect()
    }
    pub fn rollback_on_error(&self) -> bool {
        !self.no_rollback
    }
    pub fn go(&self) -> Result<()> {
        if self.at.exists() {
            if self.force {
                self.at.delete()?;
            }
        } else {
            let mut command = clap::Command::new("cargo-craft");
            for arg in Craft::command().get_arguments() {
                if arg.get_id().as_str() == "at" {
                    command = command.arg(
                        clap::Arg::new("at").value_parser(valid_manifest_path)
                    );
                } else {
                    command = command.arg(arg.clone());
                }
            }
            command.get_matches_from(Self::args());
        };
        let manifest_path = self.manifest_path();
        let manifest_string = render(&self, "Cargo.toml").unwrap().unwrap();
        manifest_path.write(&manifest_string.as_bytes()).unwrap();
        println!("wrote {}", manifest_path);

        let mut ttargets = vec![
            (
                render(&self, "lib.rs").unwrap(),
                vec![self.lib_entry("lib.rs")],
            ),
            (
                render(&self, "cli.rs").unwrap(),
                vec![self.lib_entry("cli.rs")],
            ),
            (
                render(&self, "{{package_name}}.rs").unwrap(),
                vec![self.lib_entry(format!("{}.rs", self.package_name()))],
            ),
            (
                render(&self, "errors.rs").unwrap(),
                vec![self.lib_entry("errors.rs")],
            ),
            (
                render_cli(&self).unwrap(),
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
            (render(&self, name).unwrap(), vec![Some(entry.clone())])
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
                match self.path_to(target).write(&template.as_bytes()) {
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
        cargo_add("serde -F derive", self.path(), self.verbose, self.quiet_add)?;
        cargo_add("iocore", self.path(), self.verbose, self.quiet_add)?;
        if self.cli || self.bin_entries().len() > 0 {
            cargo_add(
                "clap -F derive,env,string,unicode,wrap_help",
                self.path(),
                self.verbose,
                self.quiet_add,
            )?;
        }
        for dep in self.deps()? {
            cargo_add(&dep, self.path(), self.verbose, self.quiet_add)?;
        }
        let mut cargo_command_args = Vec::<String>::new();
        if self.offline {
            cargo_command_args.push("--offline".to_string());
        }
        if !self.verbose {
            cargo_command_args.push("--quiet".to_string());
        }
        for subcommand in ["check", "build", "test"] {
            let command = format!("cargo {} {}", subcommand, cargo_command_args.join(" "));
            match shell_command(&command, self.path(), self.verbose)? {
                0 => {}
                exit_code => {
                    let error = format!("{:#?} failed with {}", &command, exit_code);
                    if subcommand == "check" {
                        return Err(crate::Error::ShellCommandError(error));
                    } else {
                        std::process::exit(1);
                    }
                }
            }
        }
        shell_command("git init", self.path(), self.verbose)?;
        shell_command("git add .", self.path(), self.verbose)?;

        Ok(())
    }
}

impl ClapExecuter for Craft {
    fn run(args: &Craft) -> Result<()> {
        let could_rollback = args.rollback_on_error() && !args.path().try_canonicalize().exists();
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

pub fn cargo_add(
    dep: impl std::fmt::Display,
    current_dir: Path,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    let command = format!("cargo add {}{}", if quiet { " -q " } else { "" }, dep);
    shell_command(&command, current_dir, verbose)?;
    Ok(())
}
pub fn shell_command(
    command: impl std::fmt::Display,
    current_dir: impl Into<Path>,
    verbose: bool,
) -> Result<i32> {
    if verbose {
        eprintln!("{}", &command.to_string())
    }
    Ok(iocore::shell_command(command, current_dir)?)
}

#[derive(Parser, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    #[arg(value_parser = valid_crate_name)]
    pub name: String,

    #[arg(short = 'F', long)]
    pub features: Option<String>,

    #[arg(long, conflicts_with = "build")]
    pub dev: bool,

    #[arg(long)]
    pub build: bool,
}
impl std::fmt::Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut args = vec![self.name.to_string()];
        if self.dev {
            args.push("--dev".to_string());
        }
        if self.build {
            args.push("--build".to_string());
        }
        let features = self.features();
        if features.len() > 0 {
            args.push(format!("-F{}", features.join(",")));
        }
        write!(f, "{}", args.join(" "))
    }
}

impl Dependency {
    pub fn features(&self) -> Vec<String> {
        let mut features = Vec::<String>::new();
        for h in self
            .features
            .clone()
            .unwrap_or_default()
            .split(",")
            .filter(|h| h.trim().len() > 0)
        {
            features.push(h.to_string());
        }
        features
    }
    pub fn to_tera(&self) -> Table {
        let mut dep = Table::new();
        dep.insert("name".to_string(), Value::String(self.name.to_string()));
        dep.insert(
            "package_name".to_string(),
            Value::String(into_acceptable_package_name(self.name.as_str())),
        );
        dep.insert(
            "features".to_string(),
            Value::Array(
                self.features()
                    .iter()
                    .map(|h| Value::String(h.to_string()))
                    .collect(),
            ),
        );
        dep.insert("pascal_case".to_string(), Value::String(self.pascal_name()));
        dep.insert("dev".to_string(), Value::Boolean(self.dev));
        dep.insert("build".to_string(), Value::Boolean(self.build));
        dep
    }
    pub fn pascal_name(&self) -> String {
        to_pascal_case(self.name.as_str())
    }
}
#[cfg(test)]
mod test_craft {
    use crate::{tera, Craft, Dependency, Result};
    use clap::Parser;
    use iocore::{args_from_string, Path};

    fn craft_from_args(args: &str) -> Craft {
        Craft::parse_from(&args_from_string(args))
    }
    fn craft_from_name(name: &str) -> Craft {
        Craft {
            at: Path::raw(name),
            package_name: None,
            version: "0.1.0".to_string(),
            dep: Vec::new(),
            cli: true,
            bin: Vec::new(),
            lib_path: None,
            bin_path: ".".to_string(),
            value_enum: false,
            subcommands: false,
            verbose: false,
            quiet_add: true,
            offline: true,
            no_rollback: true,
            add_error_type: Vec::new(),
        }
    }
    #[test]
    fn test_craft_context() {
        let craft = craft_from_args("craft test-crate-name");
        assert_eq!(craft.crate_name(), "test-crate-name");
        assert_eq!(craft.package_name(), "test_crate_name");
        assert_eq!(craft.struct_name(), "TestCrateName");
        assert_eq!(craft.version(), "0.0.1");
        assert_eq!(craft.lib_path(), Path::raw("test-crate-name"));
    }
    #[test]
    fn test_craft_dependencies_with_features_string() -> Result<()> {
        let mut craft = craft_from_name("dependencies");
        craft.dep = vec![
            "reqwest -Fblocking,deflate".to_string(),
            "k9 --dev".to_string(),
            "clap_builder --build".to_string(),
        ];

        let dependencies = craft.deps()?;
        assert_eq!(
            dependencies,
            vec![
                Dependency {
                    name: "reqwest".to_string(),
                    features: Some("blocking,deflate".to_string()),
                    dev: false,
                    build: false,
                },
                Dependency {
                    name: "k9".to_string(),
                    features: None,
                    dev: true,
                    build: false,
                },
                Dependency {
                    name: "clap-builder".to_string(),
                    features: None,
                    dev: false,
                    build: true,
                }
            ]
        );

        assert_eq!(dependencies[0].pascal_name(), "Reqwest");
        assert_eq!(dependencies[1].pascal_name(), "K9");
        assert_eq!(dependencies[2].pascal_name(), "ClapBuilder");
        Ok(())
    }
    #[test]
    fn test_tera() -> Result<()> {
        let mut craft = craft_from_name("dependencies");
        craft.dep = vec![
            "reqwest -Fblocking,deflate".to_string(),
            "k9 --dev".to_string(),
            "clap_builder --build".to_string(),
        ];
        tera(&craft)?;
        Ok(())
    }
}

use crate::errors::{Error, ExecutionResult, Result};
use crate::helpers::{
    crate_name_from_path, extend_table, into_acceptable_error_type_name,
    into_acceptable_package_name, package_name_from_string_or_path, path_to_entry_path,
    struct_name_from_package_name, to_pascal_case, valid_crate_name, valid_manifest_path,
    valid_package_name,
};
use crate::templates::{render, render_cli, render_info_string};
use crate::traceback;
use chrono::{DateTime, Local};
use clap::CommandFactory;
use clap::Parser;
use iocore::Path;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use toml::{Table, Value};

#[derive(Parser, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct Craft {
    #[arg(
        help = "path to new directory containing new crate\n(note: use `--package-name' to define crate name instead of using `<AT>' directory name)"
    )]
    pub at: Path,

    #[arg(short = 'P', long, value_parser = valid_package_name, help = "defines crate name (optional: defaults to directory name of the AT argument)")]
    pub package_name: Option<String>,

    #[arg(long, default_value = "0.0.1")]
    pub version: String,

    #[arg(short, long)]
    pub dep: Vec<String>,

    #[arg(short, long)]
    pub cli: bool,

    #[arg(long = "bare", requires="cli", conflicts_with_all=["subcommands", "add_error_type", "value_enum", "default_bin_name", "bin"])]
    pub cli_barebones: bool,

    #[arg(
        short = 'D',
        long,
        default_value = "{{ crate_name }}",
        conflicts_with = "main"
    )]
    pub default_bin_name: String,

    #[arg(short, long)]
    pub bin: Vec<String>,

    #[arg(short, long, conflicts_with = "main")]
    pub lib_path: Option<String>,

    #[arg(long, default_value = ".", conflicts_with = "main")]
    pub bin_path: String,

    #[arg(
        short,
        long,
        help = "use 'main.rs' for default bin and 'src' for lib_path",
        conflicts_with_all=["default_bin_name"],
    )]
    pub main: bool,

    #[arg(short = 'V', long)]
    pub value_enum: bool,

    #[arg(short, long, requires = "cli")]
    pub subcommands: bool,

    #[arg(short = 'C', long = "subcommand", help="add subcommands", value_parser=valid_package_name, requires="cli")]
    pub subcommand_names: Vec<String>,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short = 'S', long)]
    pub silent: bool,

    #[arg(short, long, help = "cargo add --quiet")]
    pub quiet_add: bool,

    #[arg(long)]
    pub description: Option<String>,

    #[arg(short, long)]
    pub offline: bool,

    #[arg(short = 'e', long)]
    pub add_error_type: Vec<String>,

    #[arg(short, long)]
    pub no_rollback: bool,

    #[arg(short, long)]
    pub force: bool,

    #[arg(
        long,
        help = "prints the absolute path to the new project directory at the very end of the new project creation so that external scripts can take necessary action"
    )]
    pub script: bool,

    #[arg(skip = chrono::Local::now())]
    pub started_at: DateTime<Local>,

    #[arg(skip)]
    pub finished_at: Option<DateTime<Local>>,

    #[arg(skip)]
    pub runtime_errors: Vec<Error>,
}
pub trait ClapExecuter: Parser + std::fmt::Debug {
    fn run(args: &Self) -> Result<()>;
    fn main() -> ExecutionResult<Self> {
        let args = Self::parse_from(Self::args());
        match Self::run(&args) {
            Ok(()) => ExecutionResult::Ok(args),
            Err(error) => ExecutionResult::Err(args, error),
        }
    }
    fn args() -> Vec<String> {
        let argv = iocore::env::args();
        let execname = Path::new(&argv[0]).name();
        let shift_args = execname == "cargo";
        let args = if shift_args {
            argv[1..].to_vec()
        } else {
            argv.to_vec()
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
    pub fn subcommand_names(&self) -> Vec<String> {
        if self.subcommand_names.len() > 0 {
            self.subcommand_names.clone()
        } else {
            vec!["hello".to_string()]
        }
        .into_iter()
        .map(to_pascal_case)
        .collect()
    }
    pub fn struct_name(&self) -> String {
        struct_name_from_package_name(&self.package_name())
    }
    pub fn version(&self) -> String {
        self.version.clone()
    }
    pub fn lib_path(&self) -> Path {
        if self.main {
            self.path_to("src")
        } else {
            let lib_path_sanitized =
                crate_name_from_path(&self.lib_path.clone().unwrap_or_else(|| self.crate_name()))
                    .expect("lib-path sanitized via crate_name_from_path");
            self.path_to(lib_path_sanitized)
        }
        .relative_to_cwd()
    }
    pub fn project_path(&self) -> Path {
        self.path()
    }
    pub fn bin_path(&self) -> Path {
        if self.main {
            self.lib_path()
        } else {
            self.path_to(&self.bin_path).relative_to_cwd()
        }
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
        if !binaries.contains(&self.crate_name()) || binaries.is_empty() {
            binaries.push(
                self.default_bin_name()
                    .unwrap_or_else(|_| self.crate_name()),
            );
        }

        binaries
    }
    pub fn bin_entries(&self) -> Vec<Table> {
        let mut entries = Vec::<Table>::new();
        for name in self.bin_names() {
            let mut table = Table::new();
            table.insert("name".to_string(), Value::String(name.clone()));
            table.insert(
                "is_cargo".to_string(),
                Value::Boolean(name.starts_with("cargo-")),
            );
            table.insert(
                "path".to_string(),
                Value::String(
                    Path::new(&self.bin_path())
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
                Value::String(Path::new(&self.project_path()).join(&name).to_string()),
            );
            entries.push(table);
        }
        entries
    }
    pub fn lib_entry(&self, path: impl Display) -> Option<Table> {
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
    pub fn path_to(&self, to: impl Display) -> Path {
        self.path().join(to.to_string())
    }
    pub fn manifest_path(&self) -> Path {
        self.path_to("Cargo.toml")
    }
    pub fn default_bin_name(&self) -> Result<String> {
        if self.main {
            Ok("main".to_string())
        } else {
            Ok(render_info_string(&self.clone(), &self.default_bin_name)?)
        }
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
    pub fn render_templates(&self) -> Result<Vec<(String, Vec<Option<Table>>)>> {
        let mut ttargets = if !self.cli_barebones {
            vec![
                (
                    render(&self, "lib.rs").unwrap(),
                    vec![self.lib_entry("lib.rs")],
                ),
                (
                    render(&self, "dispatch.rs").unwrap(),
                    vec![self.lib_entry("dispatch.rs")],
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
            ]
        } else {
            vec![
                (
                    render(&self, "bare.main.rs").unwrap(),
                    vec![self.lib_entry("main.rs")],
                ),
                (
                    render(&self, "bare.mod.cli.rs").unwrap(),
                    vec![self.lib_entry("cli.rs")],
                ),
            ]
        };
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
        Ok(ttargets)
    }
    pub fn render_and_write_templates(&self) -> Result<Vec<Path>> {
        let ttargets = self.render_templates()?;
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
                let path = self.path_to(target);
                match path.write(&template.as_bytes()) {
                    Ok(path) => {
                        if !self.silent && self.verbose {
                            eprintln!("wrote {path}");
                        }
                        written_paths.push(path);
                    }
                    Err(error) => {
                        return Err(Error::IOError(format!("error writing {path}: {error}")))
                    }
                }
            }
        }
        Ok(written_paths)
    }
    pub fn rustfmt_paths(&self, written_paths: &Vec<Path>) -> Result<()> {
        for target in written_paths {
            if target.extension().unwrap_or_default().ends_with("rs") {
                self.shell_command(format!("rustfmt {}", target.relative_to_cwd()), Path::cwd())?;
            }
        }
        Ok(())
    }
    pub fn is_cli(&self) -> bool {
        self.main || self.cli
    }
    pub fn cargo_add_dependencies(&self) -> Result<()> {
        if self.is_cli() {
            self.cargo_add("clap -F derive,env,string,unicode,wrap_help", self.path())?;
        }
        self.cargo_add("iocore", self.path())?;
        self.cargo_add("serde -F derive", self.path())?;
        for dep in self.deps()? {
            self.cargo_add(&dep, self.path())?;
        }
        Ok(())
    }
    pub fn run_git_ops(&self) -> Result<()> {
        self.shell_command("git init", self.path())?;
        self.shell_command("git add .", self.path())?;
        Ok(())
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
                    command = command.arg(clap::Arg::new("at").value_parser(valid_manifest_path));
                } else {
                    command = command.arg(arg.clone());
                }
            }
            command.get_matches_from(Self::args());
        };
        let manifest_path = self.manifest_path();
        let manifest_string = render(&self, "Cargo.toml").unwrap().unwrap();
        manifest_path.write(&manifest_string.as_bytes()).unwrap();
        if !self.silent && self.verbose {
            eprintln!("wrote {}", manifest_path);
        }

        let written_paths = self.render_and_write_templates()?;
        self.rustfmt_paths(&written_paths)?;

        self.cargo_add_dependencies()?;

        self.run_git_ops()?;

        for subcommand in ["check", "build", "test"] {
            self.call_cargo_subcommand(subcommand)?;
        }

        self.write_receipt()?;
        if self.script {
            // the very last println should be crate name so that external scripts can use that information
            let name = self.at.name();
            println!("{name}");
        }
        Ok(())
    }
    pub fn write_receipt(&self) -> Result<()> {
        let mut receipt = self.clone();
        receipt.at = receipt.path().try_canonicalize();
        receipt.finished_at = Some(Local::now());
        let path = Craft::receipts_path();

        let (mut receipts, errors) = self.read_receipts(&path).unwrap_or_default();
        if errors.len() > 0 && receipts.is_empty() && !self.silent && self.verbose {
            for (location, error) in errors.iter() {
                eprintln!(
                    "[{}:{}] WARNING: trying to parse {location} in {path}: {error}",
                    file!(),
                    line!()
                );
            }
        }
        receipts.extend(receipts.clone());
        let lines = receipts
            .into_iter()
            .map(|receipt| {
                serde_json::to_string(&receipt)
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            })
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect::<Vec<String>>();

        let wrote_to = path.write(lines.join("\n").as_bytes())?;
        eprintln!("wrote receipt to: {wrote_to}");
        Ok(())
    }
    pub fn call_cargo_subcommand<T: Display + for<'a> PartialEq<&'a str>>(
        &self,
        subcommand: T,
    ) -> Result<()> {
        let mut cargo_command_args = Vec::<String>::new();
        if self.offline {
            cargo_command_args.push("--offline".to_string());
        }
        if self.quiet_add || self.silent {
            cargo_command_args.push("--quiet".to_string());
        }
        let command = format!("cargo {} {}", subcommand, cargo_command_args.join(" "));
        if !self.silent && self.verbose {
            eprintln!("{command}");
        }
        match self.shell_command(&command, self.path())? {
            0 => Ok(()),
            exit_code => {
                let error = format!("{:#?} failed with {}", &command, exit_code);
                if subcommand == "check" {
                    return Err(crate::Error::ShellCommandError(error));
                } else {
                    eprintln!("command failed: `{command}'");
                    std::process::exit(1);
                }
            }
        }
    }
}

impl ClapExecuter for Craft {
    fn run(args: &Craft) -> Result<()> {
        let mut post_run_stderr = Vec::<String>::new();
        let could_rollback = args.rollback_on_error() && !args.path().try_canonicalize().exists();
        match write_history() {
            Ok(history) => {
                let size = history.len();
                post_run_stderr.push(format!("{size} entries in history"));
            }
            Err(error) => {
                post_run_stderr.push(format!("failed to write to history: {error}"));
            }
        }
        let display_post_run_messages = move || {
            let msgcount = post_run_stderr.len().to_string();
            for (index, message) in post_run_stderr.iter().enumerate() {
                let cur = (index + 1).to_string();
                let pad = " ".repeat(msgcount.len() - cur.len());
                let progress = format!("{pad}{cur}/{msgcount}");
                eprintln!("[post-run-message {progress}] {message}");
            }
        };
        match args.go() {
            Ok(()) => {
                display_post_run_messages();
                Ok(())
            }
            Err(error) => {
                eprintln!("ERROR: {error}");
                if could_rollback {
                    eprintln!("rolling back {}", args.path());
                    args.path().delete()?;
                }
                display_post_run_messages();
                Ok(())
            }
        }
    }
}
impl Craft {
    pub fn cargo_add(&self, dep: impl Display, current_dir: Path) -> Result<()> {
        let mut opts = Vec::<String>::new();
        if self.quiet_add {
            opts.push("-q".to_string());
        }
        if self.offline {
            opts.push("--offline".to_string());
        }
        opts.push(dep.to_string());

        let command = format!("cargo add {}", opts.join(" "));
        // eprintln!("\x1b[1;38;5;34m{command}\x1b[0m");
        self.shell_command(&command, current_dir)?;
        Ok(())
    }
    pub fn shell_command(
        &self,
        command: impl Display,
        current_dir: impl Into<Path>,
    ) -> Result<i32> {
        if self.verbose {
            println!("{}", &command.to_string())
        }
        Ok(iocore::shell_command(command, current_dir)?)
    }
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

    #[arg(long)]
    pub optional: bool,
}
impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut args = vec![self.name.to_string()];
        if self.dev {
            args.push("--dev".to_string());
        }
        if self.build {
            args.push("--build".to_string());
        }
        if self.optional {
            args.push("--optional".to_string());
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
impl Craft {
    pub fn receipts_path() -> Path {
        Path::new("~/.cargo/craft-receipts.ldjson").try_canonicalize()
    }
    pub fn read_receipts(
        &self,
        receipts_path: &Path,
    ) -> Result<(Vec<Craft>, Vec<(String, serde_json::Error)>)> {
        Ok(if receipts_path.is_file() {
            let mut old_receipts = Vec::<Craft>::new();

            let lines = receipts_path.read_lines()?;
            let mut errors = Vec::<(String, serde_json::Error)>::new();
            for (index, line) in lines.iter().enumerate() {
                let lineno = index + 1;
                match serde_json::from_str::<Craft>(line.as_str()) {
                    Ok(old) => old_receipts.push(old),
                    Err(error) => {
                        errors.push((format!("json ld from line {lineno}"), error));
                    }
                }
            }
            if errors.is_empty() {
                (old_receipts, errors)
            } else if errors.len() == lines.len() {
                let all_lines = lines.join("\n");
                match serde_json::from_str::<Craft>(all_lines.as_str())
                    .map(|receipt| vec![receipt])
                    .or_else(|error| {
                        errors.insert(0, (format!("json from entire file"), error));
                        serde_json::from_str::<Vec<Craft>>(all_lines.as_str())
                    }) {
                    Ok(old_receipts) => (old_receipts, Vec::new()),
                    Err(error) => {
                        errors.push((format!("jsonld from entire file"), error));
                        (old_receipts, errors)
                    }
                }
            } else {
                (old_receipts, errors)
            }
        } else {
            return Err(Error::IOError(format!("{receipts_path} is not a file")));
        })
    }
}
fn history_path() -> Path {
    Path::new("~/.cargo/craft-history.txt").try_canonicalize()
}
pub fn write_history() -> Result<Vec<String>> {
    let mut history = history_path().read_lines().unwrap_or_default();
    let ts = chrono::Utc::now().format("%Y/%m/%d %H:%M:%S %Z");
    let args = iocore::env::args().join(" ");
    let current = format!("[{ts}] {args}");
    history.insert(0, current);
    let data = history.join("\n");
    history_path().write(data.as_bytes())?;
    Ok(history)
}

#[cfg(test)]
mod test_craft {
    use crate::{tera, Craft, Dependency, Result};
    use chrono::{Local, TimeDelta};

    use clap::Parser;
    use iocore::{args_from_string, Path};
    use iocore_test::directory_path;
    use k9::assert_equal;

    fn craft_at_test_path(name: &str) -> Path {
        directory_path!()
            .parent()
            .unwrap()
            .join("tmp")
            .join("test")
            .join(name)
    }
    fn craft_from_args(args: &str) -> Craft {
        Craft::parse_from(&args_from_string(args))
    }
    fn craft_from_name(name: &str) -> Craft {
        let at = craft_at_test_path(name);
        Craft {
            at: at,
            package_name: None,
            version: "0.1.0".to_string(),
            dep: Vec::new(),
            cli: true,
            main: false,
            bin: Vec::new(),
            lib_path: None,
            bin_path: ".".to_string(),
            value_enum: false,
            subcommands: false,
            cli_barebones: false,
            verbose: false,
            quiet_add: true,
            offline: true,
            no_rollback: true,
            default_bin_name: "main".to_string(),
            add_error_type: Vec::new(),
            force: true,
            cli_barebones: false,
            silent: false,
            started_at: Local::now(),
            finished_at: Some(Local::now() + TimeDelta::new(3600, 0)),
            runtime_errors: Vec::new(),
            description: None,
            script: true,
        }
    }
    #[test]
    fn test_craft_context() {
        let craft = craft_from_args("craft test-crate-name");
        assert_equal!(craft.crate_name(), "test-crate-name");
        assert_equal!(craft.package_name(), "test_crate_name");
        assert_equal!(craft.struct_name(), "TestCrateName");
        assert_equal!(craft.version(), "0.0.1");
        assert_equal!(craft.lib_path(), Path::raw("test-crate-name"));
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
        assert_equal!(
            dependencies,
            vec![
                Dependency {
                    name: "reqwest".to_string(),
                    features: Some("blocking,deflate".to_string()),
                    dev: false,
                    build: false,
                    optional: false,
                },
                Dependency {
                    name: "k9".to_string(),
                    features: None,
                    dev: true,
                    build: false,
                    optional: false,
                },
                Dependency {
                    name: "clap-builder".to_string(),
                    features: None,
                    dev: false,
                    build: true,
                    optional: false,
                }
            ]
        );

        assert_equal!(dependencies[0].pascal_name(), "Reqwest");
        assert_equal!(dependencies[1].pascal_name(), "K9");
        assert_equal!(dependencies[2].pascal_name(), "ClapBuilder");
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
    #[test]
    fn test_craft_paths() {
        let mut craft = craft_from_name("dummy9");
        craft.main = false;

        craft.dep = Vec::new();

        assert_equal!(craft.project_path().to_string(), "./tmp/test/dummy9");
        assert_equal!(craft.lib_path().to_string(), "dummy9");
        assert_equal!(
            craft.bin_path().to_string(),
            craft.project_path().to_string()
        );
        assert_equal!(craft.bin_names(), vec!["dummy9"]);
    }
}

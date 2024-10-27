use ::std::process::{Command, Stdio};
use cargo_craft::cli::{path_to_entry_path, Craft};
use cargo_craft::templates::{render_cli, render_errors, render_lib, render_manifest};
use clap::Parser;
use iocore::Path;
use toml::Table;

fn main() {
    let args = Craft::parse();

    let manifest_path = args.manifest_path();
    let manifest_string = render_manifest(&args);
    manifest_path.write(&manifest_string.into_bytes()).unwrap();
    println!("wrote {}", manifest_path);

    for (template, target) in vec![
        (render_lib(&args), vec![args.lib_entry()]),
        (render_errors(&args), vec![args.errors_entry()]),
        (
            render_cli(&args),
            args.bin_entries()
                .iter()
                .map(|entry| Some(entry.clone()))
                .collect::<Vec<Option<Table>>>(),
        ),
    ] {
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
    if args.bin_entries().len() > 0 {
        cargo_add("clap -F derive,env,string,unicode,wrap_help", args.path());
    }
}

fn cargo_add(dep: impl std::fmt::Display, current_dir: Path) {
    shell_command(
        format!("cargo add --target aarch64-apple-darwin {}", dep),
        current_dir,
    );
}
fn shell_command(command: String, current_dir: Path) {
    let exit_code = shell_command_opts(command.clone(), current_dir.clone())
        .expect(&format!("spawn command {:#?}", &command));
    if exit_code != 0 {
        panic!(
            "command {:#?} failed with exit code {}",
            &command, exit_code
        );
    }
}
fn shell_command_opts(command: String, current_dir: Path) -> Result<i32, String> {
    eprintln!("running {:#?} in {}", &command, &current_dir);
    let args = command
        .split(" ")
        .map(|arg| arg.trim().to_string())
        .collect::<Vec<String>>();
    let mut cmd = Command::new(args[0].clone());
    let cmd = cmd.current_dir(current_dir.to_string());
    let cmd = cmd.args(args[1..].to_vec());
    let cmd = cmd.stdin(Stdio::null());
    let cmd = cmd.stdout(Stdio::inherit());
    let cmd = cmd.stderr(Stdio::inherit());
    let mut subprocess = cmd.spawn().map_err(|e| e.to_string())?;
    Ok(subprocess
        .wait()
        .map_err(|e| e.to_string())?
        .code()
        .unwrap_or_default())
}

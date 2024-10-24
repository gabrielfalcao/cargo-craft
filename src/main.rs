use ::std::process::{Command, Stdio};
use cargo_craft::cli::{path_to_entry_path, Craft};
use cargo_craft::templates::{render_cli, render_errors, render_lib};
use clap::Parser;
use iocore::Path;
use sanitation::SString;
use std::collections::BTreeMap;
use toml::Table;

fn main() {
    let args = Craft::parse();

    let mut manifest = args.cargo_manifest();
    let manifest_string = manifest.to_string_pretty().unwrap();
    let manifest_path = args.manifest_path();
    manifest_path.write(&manifest_string.into_bytes()).unwrap();
    println!("wrote {}", manifest_path);

    manifest.set_lib_entry(args.lib_entry());
    manifest.set_bin_entries(args.bin_entries());

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
    shell_command(format!("cargo add clap -F derive,env"), args.path());
    shell_command(format!("cargo add serde"), args.path());
    shell_command(
        format!("cargo add ~/projects/work/poems.codes/tools/gadgets/packages/iocore"),
        args.path(),
    );
}

fn shell_command(command: String, current_dir: Path) -> (String, String, i32) {
    shell_command_opts(command.clone(), current_dir.clone(), None, None, None, None).expect(&format!("spawn command {:#?}", &command))
}
fn shell_command_opts(
    command: String,
    current_dir: Path,
    env_clear: Option<bool>,
    env_remove: Option<Vec<String>>,
    env: Option<BTreeMap<String, String>>,
    shell: Option<String>,
) -> Result<(String, String, i32), String> {
    let env_clear = env_clear.unwrap_or_default();
    let env_remove = env_remove.unwrap_or_default();
    let env = env.unwrap_or_default();
    let shell = shell.unwrap_or("/bin/sh".to_string());

    let mut args = vec![shell.to_string(), String::from("-c")];
    args.push(format!("\"{}\"", &command));
    let mut cmd = Command::new(&shell);
    let cmd = cmd.current_dir(current_dir.to_string());
    let cmd = cmd.args(args.clone());
    let cmd = cmd.stdin(Stdio::piped());
    let cmd = cmd.stdout(Stdio::piped());
    let mut cmd = cmd.stderr(Stdio::piped());

    if env_clear {
        cmd = cmd.env_clear();
    }
    for env in &env_remove {
        cmd = cmd.env_remove(env);
    }

    for (n, d) in env {
        cmd = cmd.env(n, d);
    }

    let mut subprocess = cmd.spawn().map_err(|e| e.to_string())?;
    let eco = subprocess
        .wait()
        .map_err(|e| e.to_string())?
        .code()
        .unwrap_or_default();

    let stderr = subprocess
        .stderr
        .map(|stderr| {
            SString::from_io_read(stderr)
                .map_err(|e| format!("{} stderr: {}", command, e))
                .unwrap_or_default()
        })
        .unwrap_or_default()
        .unchecked_safe();

    let stdout = subprocess
        .stdout
        .map(|stdout| {
            SString::from_io_read(stdout)
                .map_err(|e| format!("{} stdout: {}", command, e))
                .unwrap_or_default()
        })
        .unwrap_or_default()
        .unchecked_safe();

    Ok((stdout, stderr, eco))
}

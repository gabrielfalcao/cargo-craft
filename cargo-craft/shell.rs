use iocore::Path;
use std::process::{Command, Stdio};

pub fn shell_command(command: String, current_dir: Path) {
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

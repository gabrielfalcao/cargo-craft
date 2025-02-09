use crate::Result;
use iocore::Path;
use std::process::{Command, Stdio};

pub fn shell_command(command: String, current_dir: Path, verbose: bool) -> Result<i32> {
    if verbose {
        eprintln!("{}", &command);
    }
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
    let mut subprocess = cmd.spawn()?;
    Ok(subprocess.wait()?.code().unwrap_or_default())
}

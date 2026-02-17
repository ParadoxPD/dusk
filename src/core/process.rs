use std::ffi::OsString;
use std::io;
use std::process::{Command, ExitStatus, Stdio};

pub fn run_passthrough(bin: &str, args: &[OsString]) -> io::Result<ExitStatus> {
    Command::new(bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn run_capture(bin: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(bin).args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn command_exists(bin: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
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
    find_command_path(bin).is_some()
}

pub fn ensure_command_exists(bin: &str, used_for: &str) -> Result<(), String> {
    if command_exists(bin) {
        Ok(())
    } else {
        Err(format!(
            "required system binary `{bin}` is not available in PATH (needed for {used_for})"
        ))
    }
}

fn find_command_path(bin: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let paths = std::env::split_paths(&path_var);

    #[cfg(windows)]
    let exts: Vec<String> = std::env::var_os("PATHEXT")
        .map(|v| {
            v.to_string_lossy()
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| {
            vec![
                ".EXE".to_string(),
                ".CMD".to_string(),
                ".BAT".to_string(),
                ".COM".to_string(),
            ]
        });

    for dir in paths {
        let cand = dir.join(bin);
        if cand.is_file() {
            return Some(cand);
        }

        #[cfg(windows)]
        {
            for ext in &exts {
                let c = dir.join(format!("{bin}{ext}"));
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

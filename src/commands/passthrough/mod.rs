use std::ffi::OsString;

use crate::core::process;

pub fn run(tool: &str, args: &[OsString]) -> Result<(), String> {
    let bin = match tool {
        "find" => {
            process::ensure_command_exists("find", "dusk find passthrough")?;
            "find"
        }
        "rg" | "grep" => {
            if process::command_exists("rg") {
                "rg"
            } else if process::command_exists("grep") {
                "grep"
            } else {
                return Err(
                    "required system binary `rg` or `grep` is not available in PATH (needed for dusk rg passthrough)"
                        .to_string(),
                );
            }
        }
        _ => return Err(format!("unsupported passthrough tool: {tool}")),
    };

    let status = process::run_passthrough(bin, args).map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{bin} exited with status {status}"))
    }
}

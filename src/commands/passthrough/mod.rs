use std::ffi::OsString;

use crate::core::process;

pub fn run(tool: &str, args: &[OsString]) -> Result<(), String> {
    let bin = match tool {
        "find" => "find",
        "rg" | "grep" => {
            if process::command_exists("rg") {
                "rg"
            } else {
                "grep"
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

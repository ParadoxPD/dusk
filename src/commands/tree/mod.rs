use std::ffi::OsString;

pub fn run(args: &[OsString]) -> Result<(), String> {
    crate::commands::xtree::run(args)
}

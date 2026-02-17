use std::ffi::OsString;

pub fn run(args: &[OsString]) -> Result<(), String> {
    crate::xtree::run(args)
}

use std::ffi::OsString;

use crate::commands::fsops::{self, OpKind};

pub fn run(args: &[OsString]) -> Result<(), String> {
    fsops::run(OpKind::Move, args, false)
}

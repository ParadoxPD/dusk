use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::core::process;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OpKind {
    Move,
    Copy,
    Link,
}

impl OpKind {
    fn bin(self) -> &'static str {
        match self {
            OpKind::Move => "mv",
            OpKind::Copy => "cp",
            OpKind::Link => "ln",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            OpKind::Move => "move",
            OpKind::Copy => "copy",
            OpKind::Link => "link",
        }
    }
}

#[derive(Default)]
struct Parsed {
    force: bool,
    interactive: bool,
    no_clobber: bool,
    verbose: bool,
    recursive: bool,
    symbolic: bool,
    positional: Vec<PathBuf>,
    passthrough: Vec<OsString>,
    target_dir: Option<PathBuf>,
}

pub fn run(kind: OpKind, args: &[OsString], supports_recursive: bool) -> Result<(), String> {
    process::ensure_command_exists(kind.bin(), &format!("dusk {}", kind.bin()))?;

    if args
        .iter()
        .any(|a| matches!(a.to_string_lossy().as_ref(), "--help" | "-h" | "-?"))
    {
        return passthrough(kind.bin(), args);
    }

    let parsed = parse(kind, args, supports_recursive)?;

    if kind == OpKind::Link && parsed.positional.len() < 2 {
        return run_link_prompt(args);
    }

    if parsed.positional.len() < 2 {
        return Err(format!(
            "missing file operand (try `dusk {} --help`)",
            kind.bin()
        ));
    }

    let conflicts = detect_conflicts(&parsed);
    if !conflicts.is_empty() && parsed.no_clobber {
        if parsed.verbose {
            for c in conflicts {
                eprintln!("skip existing: {}", c.display());
            }
        }
        return Ok(());
    }

    if !conflicts.is_empty() && !parsed.force {
        let need_prompt = parsed.interactive || !conflicts.is_empty();
        if need_prompt {
            let msg = format!(
                "{} target exists for {} path(s). overwrite? [y/N] ",
                kind.noun(),
                conflicts.len()
            );
            if !confirm(&msg)? {
                return Err("operation cancelled".to_string());
            }
        }
    }

    #[cfg(unix)]
    {
        if should_offer_sudo(&parsed.positional)
            && process::command_exists("sudo")
            && !is_effective_root()
        {
            if confirm("Some paths are not owned by current user. Retry with sudo? [y/N] ")? {
                return run_with_sudo(kind.bin(), args);
            }
        }
    }

    passthrough(kind.bin(), args)
}

fn parse(kind: OpKind, args: &[OsString], supports_recursive: bool) -> Result<Parsed, String> {
    let mut p = Parsed::default();
    let mut it = args.iter().peekable();
    let mut after_double_dash = false;

    while let Some(arg) = it.next() {
        let s = arg.to_string_lossy().to_string();

        if after_double_dash {
            p.positional.push(PathBuf::from(&s));
            p.passthrough.push(arg.clone());
            continue;
        }

        if s == "--" {
            after_double_dash = true;
            p.passthrough.push(arg.clone());
            continue;
        }

        if s.starts_with("--") {
            match s.as_str() {
                "--force" => p.force = true,
                "--interactive" => p.interactive = true,
                "--no-clobber" => p.no_clobber = true,
                "--verbose" => p.verbose = true,
                "--recursive" if supports_recursive => p.recursive = true,
                "--symbolic" if kind == OpKind::Link => p.symbolic = true,
                "--target-directory" => {
                    let Some(dir) = it.next() else {
                        return Err("--target-directory requires a path".to_string());
                    };
                    p.target_dir = Some(PathBuf::from(dir));
                    p.passthrough.push(arg.clone());
                    p.passthrough.push(dir.clone());
                    continue;
                }
                _ => {}
            }
            p.passthrough.push(arg.clone());
            continue;
        }

        if s.starts_with('-') && s.len() > 1 {
            if s == "-t" {
                let Some(dir) = it.next() else {
                    return Err("-t requires a directory".to_string());
                };
                p.target_dir = Some(PathBuf::from(dir));
                p.passthrough.push(arg.clone());
                p.passthrough.push(dir.clone());
                continue;
            }
            for ch in s.chars().skip(1) {
                match ch {
                    'f' => p.force = true,
                    'i' => p.interactive = true,
                    'n' => p.no_clobber = true,
                    'v' => p.verbose = true,
                    'r' | 'R' if supports_recursive => p.recursive = true,
                    's' if kind == OpKind::Link => p.symbolic = true,
                    _ => {}
                }
            }
            p.passthrough.push(arg.clone());
            continue;
        }

        p.positional.push(PathBuf::from(&s));
        p.passthrough.push(arg.clone());
    }

    Ok(p)
}

fn detect_conflicts(p: &Parsed) -> Vec<PathBuf> {
    let mut out = Vec::new();

    if p.positional.len() < 2 {
        return out;
    }

    let (sources, dest) = if let Some(dir) = &p.target_dir {
        (&p.positional[..], dir.clone())
    } else {
        let dest = p.positional[p.positional.len() - 1].clone();
        (&p.positional[..p.positional.len() - 1], dest)
    };

    let dest_is_dir = dest.is_dir() || sources.len() > 1;

    for src in sources {
        let candidate = if dest_is_dir {
            let name = src
                .file_name()
                .map(|x| x.to_owned())
                .unwrap_or_else(|| std::ffi::OsString::from("unknown"));
            dest.join(name)
        } else {
            dest.clone()
        };
        if candidate.exists() {
            out.push(candidate);
        }
    }

    out
}

fn passthrough(bin: &str, args: &[OsString]) -> Result<(), String> {
    let status = process::run_passthrough(bin, args).map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{bin} exited with status {status}"))
    }
}

fn run_link_prompt(original_args: &[OsString]) -> Result<(), String> {
    let mut src = String::new();
    let mut dst = String::new();

    eprint!("ln source path: ");
    io::stderr().flush().map_err(|e| e.to_string())?;
    io::stdin()
        .read_line(&mut src)
        .map_err(|e| format!("failed reading source path: {e}"))?;

    eprint!("ln target path: ");
    io::stderr().flush().map_err(|e| e.to_string())?;
    io::stdin()
        .read_line(&mut dst)
        .map_err(|e| format!("failed reading target path: {e}"))?;

    let src = src.trim();
    let dst = dst.trim();
    if src.is_empty() || dst.is_empty() {
        return Err("source and target are required".to_string());
    }

    let mut args = original_args.to_vec();
    args.push(OsString::from(src));
    args.push(OsString::from(dst));
    passthrough("ln", &args)
}

fn confirm(prompt: &str) -> Result<bool, String> {
    let mut out = io::stdout().lock();
    write!(out, "{prompt}").map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("failed reading confirmation: {e}"))?;
    let answer = line.trim();
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

fn run_with_sudo(bin: &str, args: &[OsString]) -> Result<(), String> {
    let status = Command::new("sudo")
        .arg(bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("failed running sudo {bin}: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("sudo {bin} exited with status {status}"))
    }
}

#[cfg(unix)]
fn should_offer_sudo(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| !is_owned_path(p))
}

#[cfg(unix)]
fn is_owned_path(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    let uid = unsafe { libc::geteuid() };
    if let Ok(meta) = fs::symlink_metadata(path) {
        return meta.uid() == uid;
    }

    if let Some(parent) = path.parent() {
        if let Ok(meta) = fs::symlink_metadata(parent) {
            return meta.uid() == uid;
        }
    }

    true
}

#[cfg(unix)]
fn is_effective_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

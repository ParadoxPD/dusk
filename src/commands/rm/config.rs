use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Opts {
    pub force: bool,
    pub interactive: bool,
    pub recursive: bool,
    pub verbose: bool,
    pub permanent: bool,
    pub trash_tui: bool,
    pub restore: Option<String>,
    pub empty_trash: bool,
    pub paths: Vec<PathBuf>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            force: false,
            interactive: false,
            recursive: false,
            verbose: false,
            permanent: false,
            trash_tui: false,
            restore: None,
            empty_trash: false,
            paths: Vec::new(),
        }
    }
}

pub fn parse(args: &[OsString]) -> Result<Opts, String> {
    let mut opts = Opts::default();
    let mut it = args.iter().peekable();

    while let Some(arg) = it.next() {
        let s = arg.to_string_lossy();

        if s == "--" {
            for rest in it {
                opts.paths.push(PathBuf::from(rest));
            }
            break;
        }

        match s.as_ref() {
            "--help" | "-h" | "-?" => return Err("__SHOW_HELP__".to_string()),
            "--force" => {
                opts.force = true;
                continue;
            }
            "--interactive" => {
                opts.interactive = true;
                continue;
            }
            "--recursive" => {
                opts.recursive = true;
                continue;
            }
            "--verbose" => {
                opts.verbose = true;
                continue;
            }
            "--permanent" | "--hard-delete" => {
                opts.permanent = true;
                continue;
            }
            "--trash" => {
                opts.permanent = false;
                continue;
            }
            "--trash-tui" | "--scan-trash" => {
                opts.trash_tui = true;
                continue;
            }
            "--restore" => {
                let Some(query) = it.next() else {
                    return Err("--restore requires an id or pattern".to_string());
                };
                opts.restore = Some(query.to_string_lossy().to_string());
                continue;
            }
            "--empty-trash" => {
                opts.empty_trash = true;
                continue;
            }
            _ => {}
        }

        if s.starts_with('-') && s.len() > 1 {
            for ch in s[1..].chars() {
                match ch {
                    'f' => opts.force = true,
                    'i' => opts.interactive = true,
                    'r' | 'R' => opts.recursive = true,
                    'v' => opts.verbose = true,
                    'P' => opts.permanent = true,
                    _ => return Err(format!("unknown flag: -{ch}")),
                }
            }
            continue;
        }

        opts.paths.push(PathBuf::from(arg));
    }

    Ok(opts)
}

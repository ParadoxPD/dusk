use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub(super) enum SortMode {
    Name,
    Size,
    Time,
    Owner,
    Author,
    Type,
    Ext,
}

#[derive(Clone, Copy)]
pub(super) enum ColorMode {
    Auto,
    Always,
    Never,
}

pub(super) struct Opts {
    pub show_hidden: bool,
    pub almost_all: bool,
    pub long: bool,
    pub headers: bool,
    pub icons: bool,
    pub basic: bool,
    pub file_type: bool,
    pub show_author: bool,
    pub sort: SortMode,
    pub reverse: bool,
    pub human: bool,
    pub color: ColorMode,
    pub theme: Option<String>,
    pub paths: Vec<PathBuf>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            show_hidden: false,
            almost_all: false,
            long: false,
            headers: false,
            icons: true,
            basic: false,
            file_type: false,
            show_author: false,
            sort: SortMode::Name,
            reverse: false,
            human: false,
            color: ColorMode::Auto,
            theme: None,
            paths: vec![PathBuf::from(".")],
        }
    }
}

pub(super) fn parse(args: &[OsString]) -> Result<Opts, String> {
    let mut opts = Opts::default();
    opts.paths.clear();

    let mut it = args.iter().peekable();
    while let Some(arg) = it.next() {
        let s = arg.to_string_lossy();

        if s == "--" {
            for rest in it {
                opts.paths.push(PathBuf::from(rest));
            }
            break;
        }

        if let Some(v) = s.strip_prefix("--color=") {
            opts.color = match v {
                "auto" => ColorMode::Auto,
                "always" => ColorMode::Always,
                "never" => ColorMode::Never,
                _ => return Err("--color must be auto|always|never".to_string()),
            };
            continue;
        }

        if s == "--sort" {
            let Some(mode) = it.next() else {
                return Err("--sort requires a column name".to_string());
            };
            opts.sort = parse_sort(mode.to_string_lossy().as_ref())?;
            continue;
        }

        if s == "--theme" {
            let Some(name) = it.next() else {
                return Err("--theme requires a theme name".to_string());
            };
            opts.theme = Some(name.to_string_lossy().to_string());
            continue;
        }

        match s.as_ref() {
            "--help" => return Err("__SHOW_HELP__".to_string()),
            "--all" => opts.show_hidden = true,
            "--almost-all" => {
                opts.show_hidden = true;
                opts.almost_all = true;
            }
            "--long" => opts.long = true,
            "--reverse" => opts.reverse = true,
            "--human-readable" => opts.human = true,
            "--icons" => opts.icons = true,
            "--no-icons" => opts.icons = false,
            "--file-type" => opts.file_type = true,
            "--author" => opts.show_author = true,
            "--basic" => {
                opts.basic = true;
                opts.icons = false;
                opts.color = ColorMode::Never;
            }
            _ if s.starts_with('-') && s.len() > 1 => {
                for ch in s[1..].chars() {
                    match ch {
                        'a' => opts.show_hidden = true,
                        'A' => {
                            opts.show_hidden = true;
                            opts.almost_all = true;
                        }
                        'l' => opts.long = true,
                        'H' => opts.headers = true,
                        'r' => opts.reverse = true,
                        't' => opts.sort = SortMode::Time,
                        'S' => opts.sort = SortMode::Size,
                        'h' => opts.human = true,
                        '?' => return Err("__SHOW_HELP__".to_string()),
                        '1' => {}
                        _ => return Err(format!("unknown flag: -{ch}")),
                    }
                }
            }
            _ => opts.paths.push(PathBuf::from(s.to_string())),
        }
    }

    if opts.paths.is_empty() {
        opts.paths.push(PathBuf::from("."));
    }

    Ok(opts)
}

fn parse_sort(v: &str) -> Result<SortMode, String> {
    match v {
        "name" => Ok(SortMode::Name),
        "size" => Ok(SortMode::Size),
        "time" | "date" | "modified" => Ok(SortMode::Time),
        "owner" | "user" => Ok(SortMode::Owner),
        "author" => Ok(SortMode::Author),
        "type" | "kind" => Ok(SortMode::Type),
        "ext" | "extension" => Ok(SortMode::Ext),
        _ => Err("--sort must be one of: name|size|time|owner|author|type|ext".to_string()),
    }
}

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

use crate::core::devicons;
use crate::core::style::Style;
use crate::core::theme;

#[derive(Clone, Copy)]
enum SortMode {
    Name,
    Size,
    Time,
}

#[derive(Clone, Copy)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

struct Opts {
    show_hidden: bool,
    long: bool,
    icons: bool,
    basic: bool,
    sort: SortMode,
    reverse: bool,
    human: bool,
    color: ColorMode,
    theme: Option<String>,
    paths: Vec<PathBuf>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            show_hidden: false,
            long: false,
            icons: true,
            basic: false,
            sort: SortMode::Name,
            reverse: false,
            human: false,
            color: ColorMode::Auto,
            theme: None,
            paths: vec![PathBuf::from(".")],
        }
    }
}

#[derive(Clone)]
struct Row {
    perms: String,
    size: String,
    modified: String,
    display: String,
    kind: EntryKind,
}

#[derive(Clone, Copy)]
enum EntryKind {
    Dir,
    Exec,
    Link,
    File,
}

pub fn run(args: &[OsString]) -> Result<(), String> {
    if args
        .iter()
        .any(|a| matches!(a.to_string_lossy().as_ref(), "-h" | "--help"))
    {
        print_help();
        return Ok(());
    }

    let opts = parse(args)?;

    let mut style = Style::for_stdout();
    style.color = match opts.color {
        ColorMode::Auto => style.color,
        ColorMode::Always => true,
        ColorMode::Never => false,
    };
    if opts.basic {
        style.color = false;
    }

    let theme = if style.color {
        theme::resolve(opts.theme.as_deref())
    } else {
        theme::plain()
    };

    for (idx, path) in opts.paths.iter().enumerate() {
        if opts.paths.len() > 1 {
            if idx > 0 {
                println!();
            }
            println!(
                "{}",
                style.paint(theme.title, format!("{}:", path.display()))
            );
        }

        let mut entries = if path.is_file() {
            vec![path.clone()]
        } else {
            if !path.is_dir() {
                return Err(format!("no such file or directory: {}", path.display()));
            }
            fs::read_dir(path)
                .map_err(|err| format!("failed reading {}: {err}", path.display()))?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| {
                    if opts.show_hidden {
                        true
                    } else {
                        !p.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default()
                            .starts_with('.')
                    }
                })
                .collect::<Vec<_>>()
        };

        entries.sort_by(|a, b| sort_key(a, opts.sort).cmp(&sort_key(b, opts.sort)));
        if opts.reverse {
            entries.reverse();
        }

        let mut rows = Vec::new();
        for entry in &entries {
            rows.push(build_row(entry, &opts, &style)?);
        }

        print_rows(&rows, &opts, &style, theme);
    }

    Ok(())
}

fn parse(args: &[OsString]) -> Result<Opts, String> {
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
                return Err("--sort requires: name|size|time".to_string());
            };
            opts.sort = match mode.to_string_lossy().as_ref() {
                "name" => SortMode::Name,
                "size" => SortMode::Size,
                "time" => SortMode::Time,
                _ => return Err("--sort requires: name|size|time".to_string()),
            };
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
            "--all" => opts.show_hidden = true,
            "--long" => opts.long = true,
            "--reverse" => opts.reverse = true,
            "--human-readable" => opts.human = true,
            "--icons" => opts.icons = true,
            "--no-icons" => opts.icons = false,
            "--basic" => {
                opts.basic = true;
                opts.icons = false;
                opts.color = ColorMode::Never;
            }
            _ if s.starts_with('-') && s.len() > 1 => {
                for ch in s[1..].chars() {
                    match ch {
                        'a' => opts.show_hidden = true,
                        'l' => opts.long = true,
                        'r' => opts.reverse = true,
                        't' => opts.sort = SortMode::Time,
                        'S' => opts.sort = SortMode::Size,
                        'h' => opts.human = true,
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

fn print_help() {
    let theme = theme::active(None);
    println!(
        "{}dusk ls{} (ls-compatible, eza-style enhancements)\n\n{}USAGE{}\n  dusk ls [OPTIONS] [FILE|DIR]...\n\n{}COMMON FLAGS{}\n  -a, --all            Include hidden files\n  -l, --long           Long listing format\n  -r, --reverse        Reverse sort order\n  -t                   Sort by modification time\n  -S                   Sort by file size\n  -h, --human-readable Human-readable sizes in long mode\n  --color=<when>       auto | always | never\n\n{}ENHANCED FLAGS{}\n  --icons              Enable Nerd Font icons (default)\n  --no-icons           Disable icons\n  --basic              Classic plain ls output (no color, no icons)\n  --theme <name>       Select color theme\n  --sort <mode>        name | size | time\n  -h, --help           Show this help\n",
        theme.title,
        theme.reset,
        theme.accent,
        theme.reset,
        theme.accent,
        theme.reset,
        theme.accent,
        theme.reset
    );
}

fn build_row(path: &Path, opts: &Opts, style: &Style) -> Result<Row, String> {
    let md = fs::symlink_metadata(path)
        .map_err(|err| format!("failed metadata {}: {err}", path.display()))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    let kind = if md.file_type().is_symlink() {
        EntryKind::Link
    } else if md.is_dir() {
        EntryKind::Dir
    } else if is_executable(&md) {
        EntryKind::Exec
    } else {
        EntryKind::File
    };

    let icon = if opts.icons {
        match kind {
            EntryKind::Link => style.maybe_icon(devicons::ICON_LINK),
            EntryKind::Dir => style.maybe_icon(devicons::ICON_DIR),
            EntryKind::Exec => style.maybe_icon(devicons::ICON_EXEC),
            EntryKind::File => style.maybe_icon(devicons::file_icon(path)),
        }
    } else {
        ""
    };
    let gap = if icon.is_empty() { "" } else { " " };

    let mut display = format!("{icon}{gap}{name}");
    if matches!(kind, EntryKind::Dir) {
        display.push('/');
    } else if matches!(kind, EntryKind::Exec) {
        display.push('*');
    }

    let perms = permissions(&md);
    let size = if opts.human {
        human_size(md.len())
    } else {
        md.len().to_string()
    };
    let modified = md
        .modified()
        .ok()
        .map(|t| {
            let dt: DateTime<Local> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());

    Ok(Row {
        perms,
        size,
        modified,
        display,
        kind,
    })
}

fn print_rows(rows: &[Row], opts: &Opts, style: &Style, theme: theme::Theme) {
    let (perm_w, size_w, mod_w) = rows.iter().fold((0usize, 0usize, 0usize), |acc, r| {
        (
            acc.0.max(r.perms.len()),
            acc.1.max(r.size.len()),
            acc.2.max(r.modified.len()),
        )
    });

    for row in rows {
        let body_color = match row.kind {
            EntryKind::Dir => theme.accent,
            EntryKind::Exec => theme.ok,
            EntryKind::Link => theme.warn,
            EntryKind::File => theme.info,
        };

        if !opts.long {
            println!("{}", style.paint(body_color, &row.display));
            continue;
        }

        let perms = format!("{:perm_w$}", row.perms, perm_w = perm_w);
        let size = format!("{:>size_w$}", row.size, size_w = size_w);
        let modified = format!("{:mod_w$}", row.modified, mod_w = mod_w);

        let left = format!(
            "{} {} {}",
            style.paint(theme.info, perms),
            style.paint(theme.accent, size),
            style.paint(theme.number, modified)
        );
        println!("{left} {}", style.paint(body_color, &row.display));
    }
}

fn sort_key(path: &Path, sort: SortMode) -> (i128, String) {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let key = match sort {
        SortMode::Name => 0,
        SortMode::Size => fs::metadata(path).map(|m| m.len() as i128).unwrap_or(0),
        SortMode::Time => fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i128)
            .unwrap_or(0),
    };

    (key, name)
}

fn permissions(md: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = md.permissions().mode();
        let mut out = String::new();
        let flags = [
            0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
        ];
        for (i, bit) in flags.iter().enumerate() {
            let c = match i % 3 {
                0 => 'r',
                1 => 'w',
                _ => 'x',
            };
            out.push(if mode & bit != 0 { c } else { '-' });
        }
        out
    }
    #[cfg(not(unix))]
    {
        "---------".to_string()
    }
}

fn is_executable(md: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        md.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn human_size(size: u64) -> String {
    if size < 1024 {
        return format!("{size}B");
    }
    if size < 1024 * 1024 {
        return format!("{}K", size / 1024);
    }
    if size < 1024 * 1024 * 1024 {
        return format!("{}M", size / (1024 * 1024));
    }
    format!("{}G", size / (1024 * 1024 * 1024))
}

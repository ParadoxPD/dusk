use std::cmp::Reverse;
use std::fs;
use std::fs::Metadata;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

use super::config::{Config, SortMode};
use super::icons::{ICON_DIR, ICON_EXEC, ICON_LINK, file_icon};
use super::ignore::IgnoreMatcher;
use super::theme::{Theme, format_size};

pub struct TreeSummary {
    pub dir_count: usize,
    pub file_count: usize,
}

pub fn print_tree(
    root: &Path,
    cfg: &Config,
    ignore: &IgnoreMatcher,
    theme: &Theme,
) -> Result<TreeSummary, String> {
    let mut summary = TreeSummary {
        dir_count: 0,
        file_count: 0,
    };

    render_dir(root, root, "", 0, cfg, ignore, theme, &mut summary)?;
    Ok(summary)
}

fn render_dir(
    root: &Path,
    dir: &Path,
    prefix: &str,
    depth: usize,
    cfg: &Config,
    ignore: &IgnoreMatcher,
    theme: &Theme,
    summary: &mut TreeSummary,
) -> Result<(), String> {
    if let Some(max_depth) = cfg.max_depth {
        if depth >= max_depth {
            return Ok(());
        }
    }

    let mut items = fs::read_dir(dir)
        .map_err(|err| format!("failed reading {}: {err}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|path| {
            let is_dir = path.is_dir();
            !ignore.is_ignored(path, is_dir, cfg.show_hidden)
        })
        .collect::<Vec<_>>();

    if !cfg.focus_exts.is_empty() {
        items.retain(|path| {
            if path.is_dir() {
                dir_has_focus_ext(path, cfg, ignore)
            } else {
                has_focus_ext(path, cfg)
            }
        });
    }

    sort_entries(&mut items, cfg.sort_mode);

    let total = items.len();
    for (idx, path) in items.iter().enumerate() {
        let is_last = idx + 1 == total;
        let branch = if is_last { "└── " } else { "├── " };
        let next_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        let md = match fs::symlink_metadata(path) {
            Ok(md) => md,
            Err(_) => continue,
        };
        let is_symlink = md.file_type().is_symlink();

        print_single_item(root, path, &md, prefix, branch, cfg, theme)?;

        if md.is_dir() {
            summary.dir_count += 1;
            render_dir(
                root,
                path,
                &next_prefix,
                depth + 1,
                cfg,
                ignore,
                theme,
                summary,
            )?;
        } else if md.is_file() {
            summary.file_count += 1;
        } else if is_symlink {
            summary.file_count += 1;
        }
    }

    Ok(())
}

pub fn sort_key(path: &Path, mode: SortMode) -> (i64, String) {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let v = match mode {
        SortMode::Name => 0,
        SortMode::Size => fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0),
        SortMode::Time => fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    };

    (v, name)
}

fn sort_entries(items: &mut [PathBuf], mode: SortMode) {
    match mode {
        SortMode::Name => items.sort_by(|a, b| a.file_name().cmp(&b.file_name())),
        SortMode::Size | SortMode::Time => {
            items.sort_by_key(|p| Reverse(sort_key(p, mode).0));
        }
    }
}

fn print_single_item(
    root: &Path,
    path: &Path,
    md: &Metadata,
    prefix: &str,
    branch: &str,
    cfg: &Config,
    theme: &Theme,
) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    let mut info = String::new();
    if cfg.show_info {
        info.push_str(&format!(
            "{}[{}]{} ",
            theme.meta,
            metadata_str(md),
            theme.reset
        ));
    }
    if cfg.show_size && md.is_file() {
        info.push_str(&format!(
            "{}[{}]{} ",
            theme.size,
            format_size(md.len()),
            theme.reset
        ));
    }
    if cfg.highlight_big && md.is_file() && md.len() > cfg.big_threshold {
        info.push_str(&format!("{}[LARGE]{} ", theme.warn, theme.reset));
    }

    let mut icon = String::new();
    if cfg.show_icons {
        if md.is_dir() {
            icon = format!("{ICON_DIR} ");
        } else if md.file_type().is_symlink() {
            icon = format!("{ICON_LINK} ");
        } else if is_executable(md) {
            icon = format!("{ICON_EXEC} ");
        } else {
            icon = format!("{} ", file_icon(path));
        }
    }

    print!("{prefix}{}{}{}{}", theme.meta, branch, theme.reset, info);

    if md.file_type().is_symlink() {
        let target = fs::read_link(path)
            .ok()
            .map(|p| {
                if cfg.resolve_symlinks {
                    fs::canonicalize(path).unwrap_or(p).display().to_string()
                } else {
                    p.display().to_string()
                }
            })
            .unwrap_or_else(|| "<broken>".to_string());
        println!(
            "{}{}{} -> {}{}",
            theme.link, icon, name, target, theme.reset
        );
        return Ok(());
    }

    if md.is_dir() {
        let count = if cfg.show_file_count {
            format!(
                "{}[{} files]{} ",
                theme.count,
                count_files(path),
                theme.reset
            )
        } else {
            String::new()
        };

        if cfg.show_tests && is_test_name(name) {
            println!(
                "{}{}{}{}{}/{}",
                count, theme.test, icon, name, "", theme.reset
            );
        } else {
            println!("{}{}{}{}/{}", count, theme.dir, icon, name, theme.reset);
        }
        return Ok(());
    }

    if cfg.show_tests && is_test_name(name) {
        println!("{}{}{}{}", theme.test, icon, name, theme.reset);
    } else if is_executable(md) {
        println!("{}{}{}*{}", theme.exec, icon, name, theme.reset);
    } else {
        let color = file_category_color(path, theme);
        println!("{}{}{}{}", color, icon, name, theme.reset);
    }

    let _ = root;
    Ok(())
}

fn file_category_color<'a>(path: &Path, theme: &'a Theme) -> &'a str {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if matches!(
        ext.as_str(),
        "rs" | "go"
            | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "swift"
            | "kt"
            | "kts"
            | "scala"
            | "dart"
            | "lua"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "sql"
    ) {
        return theme.content; // source code / scripts
    }

    if matches!(
        ext.as_str(),
        "json" | "yaml" | "yml" | "toml" | "ini" | "conf" | "xml" | "env" | "lock"
    ) || name.starts_with(".env")
        || name == "dockerfile"
    {
        return theme.meta; // config
    }

    if matches!(
        ext.as_str(),
        "md" | "markdown" | "txt" | "rst" | "org" | "pdf" | "doc" | "docx"
    ) {
        return theme.header; // docs
    }

    if matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico"
    ) {
        return theme.count; // images/assets
    }

    if matches!(
        ext.as_str(),
        "mp3" | "wav" | "ogg" | "mp4" | "mov" | "mkv" | "avi"
    ) {
        return theme.warn; // media
    }

    if matches!(
        ext.as_str(),
        "zip" | "tar" | "gz" | "rar" | "7z" | "xz" | "bz2" | "tgz"
    ) {
        return theme.size; // archives
    }

    if matches!(
        ext.as_str(),
        "csv" | "tsv" | "parquet" | "db" | "sqlite" | "sqlite3"
    ) {
        return theme.dir; // data files
    }

    theme.file
}

fn metadata_str(md: &Metadata) -> String {
    let perm = md.permissions();
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        format!("{:o}", perm.mode() & 0o777)
    };
    #[cfg(not(unix))]
    let mode = "---".to_string();

    let modified = md
        .modified()
        .ok()
        .map(|t| {
            let dt: DateTime<Local> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());

    format!("{mode} {modified}")
}

fn count_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter(|e| e.path().is_file())
                .count()
        })
        .unwrap_or(0)
}

pub fn is_test_name(name: &str) -> bool {
    name.contains("_test.")
        || name.contains(".test.")
        || name.contains("Test.")
        || name.contains("Spec.")
        || name.contains(".spec.")
}

pub fn has_focus_ext(path: &Path, cfg: &Config) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    cfg.focus_exts.iter().any(|e| *e == ext)
}

fn dir_has_focus_ext(dir: &Path, cfg: &Config, ignore: &IgnoreMatcher) -> bool {
    let walker = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), cfg.show_hidden));
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && has_focus_ext(path, cfg) {
            return true;
        }
    }
    false
}

pub fn should_cat(path: &Path, cfg: &Config) -> bool {
    if cfg.cat_exts.is_empty() {
        return false;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    cfg.cat_exts.iter().any(|x| x == &ext)
}

pub fn print_grep(path: &Path, pattern: &str, theme: &Theme) {
    if !path.is_file() {
        return;
    }
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let reader = BufReader::new(file);
    let mut matches = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            if line.contains(pattern) {
                matches.push(format!("{}:{}", idx + 1, line));
                if matches.len() >= 5 {
                    break;
                }
            }
        }
    }

    if !matches.is_empty() {
        println!(
            "{}    ╭── matches in {} ──{}",
            theme.content,
            path.display(),
            theme.reset
        );
        for m in matches {
            println!("{}    │ {}{}", theme.content, m, theme.reset);
        }
        println!("{}    ╰────────────────{}", theme.content, theme.reset);
    }
}

pub fn print_file_content(path: &Path, cfg: &Config, theme: &Theme) {
    if !path.is_file() {
        return;
    }
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    println!(
        "{}    ╭── content of {} ──{}",
        theme.content,
        path.display(),
        theme.reset
    );

    let reader = BufReader::new(file);
    for (idx, line) in reader.lines().enumerate() {
        if !cfg.no_clip && idx >= cfg.clip {
            break;
        }
        if let Ok(line) = line {
            println!("{}    │ {}{}", theme.content, line, theme.reset);
        }
    }
    println!("{}    ╰────────────────────{}", theme.content, theme.reset);
}

pub fn walk_files<F>(
    root: &Path,
    cfg: &Config,
    ignore: &IgnoreMatcher,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(&Path),
{
    let walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), cfg.show_hidden));
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            f(path);
        }
    }
    Ok(())
}

fn is_executable(md: &Metadata) -> bool {
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

pub fn md_line_for(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let md = fs::metadata(path).ok();
    let size = md
        .as_ref()
        .map(|m| format_size(m.len()))
        .unwrap_or_else(|| "0B".to_string());

    if path.is_dir() {
        format!("- 📁 **{name}/**")
    } else {
        format!("- 📄 `{name}` _({size})_")
    }
}

pub fn json_node(path: &Path, cfg: &Config, ignore: &IgnoreMatcher) -> serde_json::Value {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    if path.is_dir() {
        let mut children = Vec::new();
        if cfg.max_depth != Some(0) {
            let mut items = fs::read_dir(path)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| !ignore.is_ignored(p, p.is_dir(), cfg.show_hidden))
                .collect::<Vec<_>>();
            sort_entries(&mut items, cfg.sort_mode);
            for item in items {
                children.push(json_node(&item, cfg, ignore));
            }
        }
        serde_json::json!({
            "name": name,
            "type": "directory",
            "path": path.display().to_string(),
            "children": children
        })
    } else {
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        serde_json::json!({
            "name": name,
            "type": "file",
            "path": path.display().to_string(),
            "size": size
        })
    }
}

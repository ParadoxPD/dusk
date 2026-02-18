use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::config::Config;
use super::ignore::IgnoreMatcher;
use super::theme::{Theme, format_size};

#[derive(Default)]
pub struct Stats {
    pub lang_stats: HashMap<String, usize>,
    pub loc_stats: HashMap<String, u64>,
    pub total_size: u64,
    pub total_loc: u64,
    pub dir_count: usize,
    pub file_count: usize,
}

pub fn collect_stats(root: &Path, cfg: &Config, ignore: &IgnoreMatcher) -> Result<Stats, String> {
    let mut stats = Stats::default();
    let walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), cfg.show_hidden));
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            stats.dir_count += 1;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        stats.file_count += 1;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("no-ext")
            .to_ascii_lowercase();
        *stats.lang_stats.entry(ext).or_insert(0) += 1;
        let ext_key = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("no-ext")
            .to_ascii_lowercase();
        let loc = count_lines(path);
        *stats.loc_stats.entry(ext_key).or_insert(0) += loc;
        stats.total_loc += loc;

        if let Ok(md) = fs::metadata(path) {
            stats.total_size += md.len();
        }
    }
    Ok(stats)
}

pub fn print_stats(stats: &Stats, theme: &Theme) {
    println!();
    println!("{}=== Language Statistics ==={}", theme.header, theme.reset);
    println!(
        "{}Total LOC: {}{}",
        theme.meta, stats.total_loc, theme.reset
    );
    println!(
        "{}Total Files: {}{}",
        theme.meta, stats.file_count, theme.reset
    );
    println!();
    let mut v = stats
        .lang_stats
        .iter()
        .map(|(ext, count)| (ext.clone(), *count))
        .collect::<Vec<_>>();
    v.sort_by(|a, b| b.1.cmp(&a.1));

    for (ext, count) in v {
        let loc = stats.loc_stats.get(&ext).copied().unwrap_or(0);
        println!(
            "{}  .{}: {} files, {} LOC{}",
            theme.meta, ext, count, loc, theme.reset
        );
    }
}

pub fn loc_for_extensions(stats: &Stats, exts: &[String]) -> u64 {
    if exts.is_empty() {
        return 0;
    }
    exts.iter()
        .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
        .map(|ext| stats.loc_stats.get(&ext).copied().unwrap_or(0))
        .sum()
}

pub type DuplicateMap = HashMap<String, Vec<PathBuf>>;

pub fn collect_duplicates(
    root: &Path,
    cfg: &Config,
    ignore: &IgnoreMatcher,
) -> Result<DuplicateMap, String> {
    let mut map: DuplicateMap = HashMap::new();

    let walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), cfg.show_hidden));
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let digest = md5::compute(bytes);
        let hash = format!("{:x}", digest);
        map.entry(hash).or_default().push(path.to_path_buf());
    }

    Ok(map)
}

pub fn print_duplicates(dupes: &DuplicateMap, cfg: &Config, theme: &Theme) {
    println!();
    println!(
        "{}=== Duplicate Files (by content) ==={}",
        theme.header, theme.reset
    );
    println!();

    let mut any = false;
    for (hash, files) in dupes {
        if files.len() <= 1 {
            continue;
        }
        any = true;
        println!(
            "{}Hash: {} ({} files){}",
            theme.warn,
            hash,
            files.len(),
            theme.reset
        );
        for file in files {
            let size = fs::metadata(file).map(|m| m.len()).unwrap_or(0);
            let rel = file
                .strip_prefix(&cfg.target_dir)
                .ok()
                .unwrap_or(file)
                .display()
                .to_string();
            println!(
                "{}  - {} ({}){}",
                theme.file,
                rel,
                format_size(size),
                theme.reset
            );
        }
        println!();
    }

    if !any {
        println!("{}No duplicate files found.{}", theme.meta, theme.reset);
    }
}

pub fn grouped_view(
    root: &Path,
    cfg: &Config,
    ignore: &IgnoreMatcher,
    theme: &Theme,
) -> Result<(), String> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), cfg.show_hidden));
    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("no-ext")
            .to_ascii_lowercase();
        groups.entry(ext).or_default().push(path.to_path_buf());
    }

    println!("{}{}{}/{}", theme.dir, root.display(), "", theme.reset);
    println!();

    let mut keys = groups.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    for ext in keys {
        let files = groups.get(&ext).cloned().unwrap_or_default();
        println!(
            "{}[{}] ({} files){}",
            theme.header,
            ext,
            files.len(),
            theme.reset
        );
        for file in files {
            println!("{}  {}{}", theme.file, file.display(), theme.reset);
        }
        println!();
    }

    Ok(())
}

pub fn print_fingerprint(
    root: &Path,
    _cfg: &Config,
    theme: &Theme,
    ignore: &IgnoreMatcher,
    stats: &Stats,
) -> Result<(), String> {
    println!("{}=== Project Fingerprint ==={}", theme.header, theme.reset);
    println!();
    println!(
        "{}📂 Directory: {}{}",
        theme.meta,
        root.display(),
        theme.reset
    );
    println!(
        "{}📁 Total Directories: {}{}",
        theme.meta, stats.dir_count, theme.reset
    );
    println!(
        "{}📄 Total Files: {}{}",
        theme.meta, stats.file_count, theme.reset
    );
    println!(
        "{}💾 Total Size: {}{}",
        theme.meta,
        format_size(stats.total_size),
        theme.reset
    );
    println!(
        "{}🧾 Total LOC: {}{}",
        theme.meta, stats.total_loc, theme.reset
    );

    let mut max_depth = 0usize;
    let fp_walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), false));
    for entry in fp_walker.filter_map(Result::ok) {
        let path = entry.path();
        let depth = path
            .strip_prefix(root)
            .ok()
            .map(|p| p.components().count())
            .unwrap_or(0);
        max_depth = max_depth.max(depth);
    }
    println!("{}📊 Max Depth: {}{}", theme.meta, max_depth, theme.reset);

    print_stats(stats, theme);

    if Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        println!();
        println!("{}=== Git Status ==={}", theme.header, theme.reset);
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!("{}🌿 Branch: {}{}", theme.meta, branch, theme.reset);

        let commits = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "0".to_string());
        println!("{}📝 Commits: {}{}", theme.meta, commits, theme.reset);
    }

    println!();
    println!(
        "{}=== Largest Files (Top 10) ==={}",
        theme.header, theme.reset
    );

    let mut files = Vec::new();
    let largest_walker = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !ignore.is_ignored(e.path(), e.file_type().is_dir(), false));
    for entry in largest_walker.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        files.push((size, path.to_path_buf()));
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    for (size, path) in files.into_iter().take(10) {
        println!(
            "{}  {} {}{}",
            theme.size,
            format_size(size),
            path.display(),
            theme.reset
        );
    }

    Ok(())
}

fn count_lines(path: &Path) -> u64 {
    match fs::read(path) {
        Ok(bytes) => {
            if bytes.is_empty() {
                return 0;
            }
            let mut lines = bytes.iter().filter(|&&b| b == b'\n').count() as u64;
            if *bytes.last().unwrap_or(&b'\n') != b'\n' {
                lines += 1;
            }
            lines
        }
        Err(_) => 0,
    }
}

pub fn audit_file(path: &Path, theme: &Theme) {
    let md = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = md.permissions().mode();
        if mode & 0o002 != 0 {
            println!(
                "{}⚠ World-writable: {}{}",
                theme.warn,
                path.display(),
                theme.reset
            );
        }
        if mode & 0o111 != 0 {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            let expected = ["sh", "bash", "py", "rb", "pl", "js", "ts", "go", "rs"];
            if !expected.contains(&ext) {
                println!(
                    "{}⚠ Suspicious executable: {}{}",
                    theme.warn,
                    path.display(),
                    theme.reset
                );
            }
        }
    }

    let content = fs::read_to_string(path)
        .unwrap_or_default()
        .to_ascii_lowercase();
    for token in ["password", "secret", "api_key", "token", "private_key"] {
        if content.contains(token) {
            println!(
                "{}⚠ Possible secret in: {}{}",
                theme.warn,
                path.display(),
                theme.reset
            );
            break;
        }
    }
}

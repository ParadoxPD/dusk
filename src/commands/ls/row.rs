use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

use crate::core::devicons;
use crate::core::style::Style;

use super::config::Opts;

#[derive(Clone)]
pub(super) struct Row {
    pub perms: String,
    pub owner: String,
    pub author: String,
    pub size: String,
    pub size_bytes: u64,
    pub modified: String,
    pub mtime_epoch: i128,
    pub display: String,
    pub kind: EntryKind,
    pub sort_name: String,
    pub sort_ext: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum EntryKind {
    Dir,
    Exec,
    Link,
    File,
}

pub(super) fn build_row(path: &Path, opts: &Opts, style: &Style) -> Result<Row, String> {
    let md = fs::symlink_metadata(path)
        .map_err(|err| format!("failed metadata {}: {err}", path.display()))?;
    let name = special_name(path).unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string()
    });

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
    match kind {
        EntryKind::Dir => display.push('/'),
        EntryKind::Link if opts.file_type => display.push('@'),
        EntryKind::Exec if !opts.file_type => display.push('*'),
        _ => {}
    }

    let perms = permissions(&md);
    let owner = owner_name(&md);
    let author = author_name(&md);
    let size = if opts.human {
        human_size(md.len())
    } else {
        md.len().to_string()
    };
    let mtime_epoch = md
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i128)
        .unwrap_or(0);
    let modified = md
        .modified()
        .ok()
        .map(|t| {
            let dt: DateTime<Local> = t.into();
            dt.format("%d %b %H:%M").to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());

    let sort_ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    Ok(Row {
        perms,
        owner,
        author,
        size,
        size_bytes: md.len(),
        modified,
        mtime_epoch,
        display,
        kind,
        sort_name: name.to_ascii_lowercase(),
        sort_ext,
    })
}

fn special_name(path: &Path) -> Option<String> {
    let s = path.to_string_lossy();
    if s.ends_with("/.") || s == "." {
        Some(".".to_string())
    } else if s.ends_with("/..") || s == ".." {
        Some("..".to_string())
    } else {
        None
    }
}

pub(super) fn permissions(md: &fs::Metadata) -> String {
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

pub(super) fn is_executable(md: &fs::Metadata) -> bool {
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

pub(super) fn human_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "K", "M", "G", "T", "P"];
    let mut value = size as f64;
    let mut idx = 0usize;
    while value >= 1024.0 && idx + 1 < UNITS.len() {
        value /= 1024.0;
        idx += 1;
    }

    if idx == 0 {
        format!("{size}B")
    } else if value >= 10.0 {
        format!("{value:.1}{}", UNITS[idx])
    } else {
        format!("{value:.2}{}", UNITS[idx])
    }
}

fn owner_name(md: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::ffi::CStr;
        use std::os::unix::fs::MetadataExt;

        let uid = md.uid();
        unsafe {
            let pwd = libc::getpwuid(uid);
            if pwd.is_null() {
                return uid.to_string();
            }
            CStr::from_ptr((*pwd).pw_name)
                .to_string_lossy()
                .into_owned()
        }
    }
    #[cfg(not(unix))]
    {
        "-".to_string()
    }
}

fn author_name(md: &fs::Metadata) -> String {
    // Portable fallback: same as owner.
    owner_name(md)
}

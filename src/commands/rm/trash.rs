use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

#[derive(Clone, Debug)]
pub struct TrashItem {
    pub id: String,
    pub name: String,
    pub trash_path: PathBuf,
    pub meta_path: PathBuf,
    pub original_path: PathBuf,
    pub deleted_at_unix: u64,
}

pub fn trash_root() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("DUSK_TRASH_DIR") {
        if path.trim().is_empty() {
            return Err("DUSK_TRASH_DIR is set but empty".to_string());
        }
        return Ok(PathBuf::from(path));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data).join("dusk").join("Trash"));
        }
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            return Ok(PathBuf::from(user_profile)
                .join("AppData")
                .join("Local")
                .join("dusk")
                .join("Trash"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = home_dir() {
            return Ok(home.join(".Trash").join("dusk"));
        }
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.trim().is_empty() {
            return Ok(PathBuf::from(xdg).join("Trash").join("dusk"));
        }
    }

    if let Some(home) = home_dir() {
        return Ok(home.join(".local").join("share").join("Trash").join("dusk"));
    }

    Err("unable to resolve trash directory".to_string())
}

fn home_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.trim().is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    None
}

pub fn ensure_layout(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join("files"))
        .map_err(|e| format!("failed creating trash files dir: {e}"))?;
    fs::create_dir_all(root.join("meta"))
        .map_err(|e| format!("failed creating trash meta dir: {e}"))?;
    Ok(())
}

pub fn move_to_trash(path: &Path) -> Result<TrashItem, String> {
    let root = trash_root()?;
    ensure_layout(&root)?;

    let abs_original = absolute_path(path)?;
    let base_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let id = unique_id();
    let mut destination = root.join("files").join(format!("{id}__{base_name}"));
    if destination.exists() {
        destination = root
            .join("files")
            .join(format!("{id}__{}", sanitize_name(&base_name)));
    }

    move_path(path, &destination)?;

    let deleted_at_unix = now_unix();
    let meta_path = root.join("meta").join(format!("{id}.json"));
    let meta = json!({
        "id": id,
        "name": base_name,
        "original_path": abs_original,
        "deleted_at_unix": deleted_at_unix,
        "trash_path": destination,
    });

    fs::write(&meta_path, meta.to_string())
        .map_err(|e| format!("failed writing trash metadata {}: {e}", meta_path.display()))?;

    read_item_from_meta(&meta_path)
}

pub fn list_trash() -> Result<Vec<TrashItem>, String> {
    let root = trash_root()?;
    ensure_layout(&root)?;

    let meta_dir = root.join("meta");
    let mut out = Vec::new();
    let read_dir = fs::read_dir(&meta_dir)
        .map_err(|e| format!("failed reading trash metadata {}: {e}", meta_dir.display()))?;

    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        if let Ok(item) = read_item_from_meta(&path) {
            out.push(item);
        }
    }

    out.sort_by_key(|x| std::cmp::Reverse(x.deleted_at_unix));
    Ok(out)
}

pub fn restore(item: &TrashItem) -> Result<PathBuf, String> {
    if !item.trash_path.exists() {
        remove_meta_if_exists(&item.meta_path)?;
        return Err(format!(
            "trash object missing for {}",
            item.original_path.display()
        ));
    }

    if let Some(parent) = item.original_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed creating restore path {}: {e}", parent.display()))?;
    }

    let target = next_available_restore_path(&item.original_path);
    move_path(&item.trash_path, &target)?;
    remove_meta_if_exists(&item.meta_path)?;
    Ok(target)
}

pub fn purge(item: &TrashItem) -> Result<(), String> {
    if item.trash_path.exists() {
        hard_delete(&item.trash_path, true)?;
    }
    remove_meta_if_exists(&item.meta_path)
}

pub fn hard_delete(path: &Path, recursive: bool) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            let ty = meta.file_type();
            if ty.is_dir() {
                if recursive {
                    fs::remove_dir_all(path)
                        .map_err(|e| format!("failed to remove {}: {e}", path.display()))
                } else {
                    fs::remove_dir(path)
                        .map_err(|e| format!("failed to remove {}: {e}", path.display()))
                }
            } else {
                fs::remove_file(path)
                    .map_err(|e| format!("failed to remove {}: {e}", path.display()))
            }
        }
        Err(e) => Err(format!("failed stat {}: {e}", path.display())),
    }
}

fn remove_meta_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!(
            "failed removing trash metadata {}: {err}",
            path.display()
        )),
    }
}

fn read_item_from_meta(meta_path: &Path) -> Result<TrashItem, String> {
    let raw = fs::read_to_string(meta_path)
        .map_err(|e| format!("failed reading metadata {}: {e}", meta_path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("invalid metadata {}: {e}", meta_path.display()))?;

    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let original_path = PathBuf::from(
        value
            .get("original_path")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );
    let trash_path = PathBuf::from(
        value
            .get("trash_path")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );
    let deleted_at_unix = value
        .get("deleted_at_unix")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if id.is_empty() || trash_path.as_os_str().is_empty() {
        return Err(format!("invalid metadata in {}", meta_path.display()));
    }

    Ok(TrashItem {
        id,
        name,
        trash_path,
        meta_path: meta_path.to_path_buf(),
        original_path,
        deleted_at_unix,
    })
}

fn unique_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos}-{}", std::process::id())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(|e| format!("failed resolving current dir: {e}"))
            .map(|cwd| cwd.join(path))
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c == '/' || c == '\\' { '_' } else { c })
        .collect()
}

fn next_available_restore_path(original: &Path) -> PathBuf {
    if !original.exists() {
        return original.to_path_buf();
    }

    let parent = original.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = original
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "restored".to_string());
    let ext = original
        .extension()
        .map(|e| e.to_string_lossy().to_string());

    for idx in 1..=9999 {
        let file = if let Some(ext) = &ext {
            format!("{stem}.restored-{idx}.{ext}")
        } else {
            format!("{stem}.restored-{idx}")
        };
        let candidate = parent.join(file);
        if !candidate.exists() {
            return candidate;
        }
    }

    parent.join(format!("{stem}.restored"))
}

fn move_path(src: &Path, dst: &Path) -> Result<(), String> {
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::CrossesDevices => {
            copy_path(src, dst)?;
            hard_delete(src, true)
        }
        Err(err) => Err(format!(
            "failed moving {} -> {}: {err}",
            src.display(),
            dst.display()
        )),
    }
}

fn copy_path(src: &Path, dst: &Path) -> Result<(), String> {
    let meta =
        fs::symlink_metadata(src).map_err(|e| format!("failed stat {}: {e}", src.display()))?;

    if meta.file_type().is_dir() {
        fs::create_dir_all(dst).map_err(|e| format!("failed creating {}: {e}", dst.display()))?;
        for entry in
            fs::read_dir(src).map_err(|e| format!("failed reading {}: {e}", src.display()))?
        {
            let entry = entry.map_err(|e| format!("failed reading {}: {e}", src.display()))?;
            let name = entry.file_name();
            copy_path(&entry.path(), &dst.join(name))?;
        }
        return Ok(());
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed creating {}: {e}", parent.display()))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if meta.file_type().is_symlink() {
            let target = fs::read_link(src)
                .map_err(|e| format!("failed reading symlink {}: {e}", src.display()))?;
            std::os::unix::fs::symlink(target, dst)
                .map_err(|e| format!("failed creating symlink {}: {e}", dst.display()))?;
            return Ok(());
        }
        if meta.file_type().is_block_device()
            || meta.file_type().is_char_device()
            || meta.file_type().is_fifo()
            || meta.file_type().is_socket()
        {
            return Err(format!(
                "unsupported special file for trash move: {}",
                src.display()
            ));
        }
    }

    fs::copy(src, dst)
        .map_err(|e| format!("failed copying {} -> {}: {e}", src.display(), dst.display()))?;
    Ok(())
}

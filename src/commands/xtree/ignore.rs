use std::fs;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use super::config::Config;

const COMMON_IGNORES: [&str; 15] = [
    "node_modules",
    ".git",
    "dist",
    "build",
    "target",
    ".next",
    ".nuxt",
    "out",
    "coverage",
    ".cache",
    ".venv",
    "venv",
    "__pycache__",
    "*.pyc",
    ".DS_Store",
];

pub struct IgnoreMatcher {
    root: PathBuf,
    user_globs: GlobSet,
    tree_globs: GlobSet,
    gitignore: Option<Gitignore>,
    use_gitignore: bool,
    use_treeignore: bool,
}

impl IgnoreMatcher {
    pub fn new(root: &Path, cfg: &Config) -> Result<Self, String> {
        let mut user_builder = GlobSetBuilder::new();
        for common in COMMON_IGNORES {
            let glob = Glob::new(common).map_err(|err| err.to_string())?;
            user_builder.add(glob);
        }
        for pattern in &cfg.excludes {
            let glob = Glob::new(pattern)
                .or_else(|_| Glob::new(&format!("*{pattern}*")))
                .map_err(|err| format!("invalid exclude pattern `{pattern}`: {err}"))?;
            user_builder.add(glob);
        }

        let mut tree_builder = GlobSetBuilder::new();
        let treeignore = root.join(".treeignore");
        if treeignore.is_file() {
            let content = fs::read_to_string(&treeignore).unwrap_or_default();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let glob = Glob::new(trimmed)
                    .or_else(|_| Glob::new(&format!("*{trimmed}*")))
                    .map_err(|err| format!("invalid .treeignore pattern `{trimmed}`: {err}"))?;
                tree_builder.add(glob);
            }
        }

        let gitignore = if cfg.use_gitignore {
            let path = root.join(".gitignore");
            if path.is_file() {
                let mut builder = GitignoreBuilder::new(root);
                if let Some(err) = builder.add(path) {
                    return Err(format!("failed loading .gitignore: {err}"));
                }
                Some(
                    builder
                        .build()
                        .map_err(|err| format!("failed parsing .gitignore: {err}"))?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            root: root.to_path_buf(),
            user_globs: user_builder.build().map_err(|err| err.to_string())?,
            tree_globs: tree_builder.build().map_err(|err| err.to_string())?,
            gitignore,
            use_gitignore: cfg.use_gitignore,
            use_treeignore: cfg.use_treeignore,
        })
    }

    pub fn is_ignored(&self, path: &Path, is_dir: bool, show_hidden: bool) -> bool {
        let rel = path.strip_prefix(&self.root).unwrap_or(path);
        if rel.as_os_str().is_empty() {
            return false;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !show_hidden && name.starts_with('.') && name != "." {
            return true;
        }

        if matches!(
            name,
            "node_modules"
                | ".git"
                | "dist"
                | "build"
                | "target"
                | ".next"
                | ".nuxt"
                | "out"
                | "coverage"
                | ".cache"
                | ".venv"
                | "venv"
                | "__pycache__"
                | ".DS_Store"
                | "Thumbs.db"
        ) {
            return true;
        }
        if name.ends_with(".pyc") {
            return true;
        }

        if self.user_globs.is_match(rel) || self.user_globs.is_match(path) {
            return true;
        }

        if self.use_treeignore && (self.tree_globs.is_match(rel) || self.tree_globs.is_match(path))
        {
            return true;
        }

        if self.use_gitignore {
            if let Some(gitignore) = &self.gitignore {
                let m = gitignore.matched_path_or_any_parents(rel, is_dir);
                if m.is_ignore() {
                    return true;
                }
            }
        }

        false
    }
}

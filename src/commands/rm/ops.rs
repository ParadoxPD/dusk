use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::core::style::Style;
use crate::core::theme;

use super::config::Opts;
use super::trash;

pub fn run(opts: &Opts) -> Result<(), String> {
    if opts.paths.is_empty() {
        if opts.force {
            return Ok(());
        }
        return Err("missing operand (try `dusk rm --help`)".to_string());
    }

    let style = Style::for_stdout();
    let t = theme::active(None);

    let mut had_error = false;
    for path in &opts.paths {
        if let Err(err) = remove_one(path, opts, &style, t) {
            had_error = true;
            if opts.force {
                continue;
            }
            return Err(err);
        }
    }

    if had_error {
        return Err("one or more removals failed".to_string());
    }
    Ok(())
}

fn remove_one(path: &Path, opts: &Opts, style: &Style, t: theme::Theme) -> Result<(), String> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            if opts.force {
                return Ok(());
            }
            return Err(format!(
                "cannot remove '{}': No such file or directory",
                path.display()
            ));
        }
        Err(err) => {
            return Err(format!("cannot remove '{}': {err}", path.display()));
        }
    };

    if meta.is_dir() && !opts.recursive {
        return Err(format!(
            "cannot remove '{}': Is a directory (use -r)",
            path.display()
        ));
    }

    if opts.interactive && !prompt_confirm(path)? {
        if opts.verbose {
            println!(
                "{}",
                style.paint(t.subtle, format!("skipped {}", path.display()))
            );
        }
        return Ok(());
    }

    if opts.permanent {
        trash::hard_delete(path, opts.recursive)?;
        if opts.verbose {
            println!(
                "{}",
                style.paint(t.warn, format!("deleted permanently {}", path.display()))
            );
        }
    } else {
        let item = trash::move_to_trash(path)?;
        if opts.verbose {
            println!(
                "{}",
                style.paint(
                    t.ok,
                    format!(
                        "trashed {} -> {}",
                        path.display(),
                        item.trash_path.display()
                    )
                )
            );
        }
    }

    Ok(())
}

fn prompt_confirm(path: &Path) -> Result<bool, String> {
    let mut out = io::stdout().lock();
    write!(out, "remove '{}'? [y/N] ", path.display()).map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("failed reading confirmation: {e}"))?;
    let trimmed = line.trim();
    Ok(trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes"))
}

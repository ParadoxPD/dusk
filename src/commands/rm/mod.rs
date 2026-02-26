use std::ffi::OsString;
use std::io::{self, Write};

mod config;
mod help;
mod ops;
mod trash;
mod tui;

pub fn run(args: &[OsString]) -> Result<(), String> {
    let opts = match config::parse(args) {
        Ok(v) => v,
        Err(e) if e == "__SHOW_HELP__" => {
            help::print_help();
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    if opts.trash_tui {
        return tui::run();
    }

    if let Some(query) = opts.restore.as_deref() {
        return restore_matching(query, opts.force);
    }

    if opts.empty_trash {
        return empty_trash(opts.force);
    }

    ops::run(&opts)
}

fn restore_matching(query: &str, force: bool) -> Result<(), String> {
    let items = trash::list_trash()?;
    let mut matches = items
        .into_iter()
        .filter(|item| {
            item.id.starts_with(query)
                || item.name.contains(query)
                || item.original_path.to_string_lossy().contains(query)
        })
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(format!("no trash entries matched `{query}`"));
    }

    if !force && matches.len() > 1 {
        let msg = format!(
            "restore {} matching entries for `{query}`? [y/N] ",
            matches.len()
        );
        if !confirm(&msg)? {
            return Err("restore cancelled".to_string());
        }
    }

    let mut restored = 0usize;
    for item in matches.drain(..) {
        trash::restore(&item)?;
        restored += 1;
    }
    println!(
        "restored {restored} entr{}",
        if restored == 1 { "y" } else { "ies" }
    );
    Ok(())
}

fn empty_trash(force: bool) -> Result<(), String> {
    let items = trash::list_trash()?;
    if items.is_empty() {
        println!("trash is already empty");
        return Ok(());
    }

    if !force {
        let msg = format!(
            "permanently delete {} trash entr{}? [y/N] ",
            items.len(),
            if items.len() == 1 { "y" } else { "ies" }
        );
        if !confirm(&msg)? {
            return Err("empty-trash cancelled".to_string());
        }
    }

    let total = items.len();
    for item in items {
        trash::purge(&item)?;
    }
    println!(
        "deleted {total} entr{} from trash",
        if total == 1 { "y" } else { "ies" }
    );
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool, String> {
    let mut out = io::stdout().lock();
    write!(out, "{prompt}").map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| format!("failed reading confirmation: {e}"))?;
    let v = line.trim();
    Ok(v.eq_ignore_ascii_case("y") || v.eq_ignore_ascii_case("yes"))
}

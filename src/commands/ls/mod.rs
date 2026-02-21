use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::core::style::Style;
use crate::core::theme;

mod config;
mod output;
mod row;

use config::{ColorMode, Opts, parse};
use output::{print_rows, sort_rows};
use row::build_row;

pub fn run(args: &[OsString]) -> Result<(), String> {
    let opts = match parse(args) {
        Ok(o) => o,
        Err(e) if e == "__SHOW_HELP__" => {
            print_help();
            return Ok(());
        }
        Err(e) => return Err(e),
    };

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

        let entries = collect_entries(path, &opts)?;

        let mut rows = Vec::new();
        for entry in &entries {
            match build_row(entry, &opts, &style) {
                Ok(row) => rows.push(row),
                Err(err) if err.contains("Permission denied") => continue,
                Err(err) => return Err(err),
            }
        }

        sort_rows(&mut rows, opts.sort, opts.reverse);
        print_rows(&rows, &opts, &style, theme);
    }

    Ok(())
}

fn collect_entries(path: &Path, opts: &Opts) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Err(format!("no such file or directory: {}", path.display()));
    }

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(err) if err.kind() == ErrorKind::PermissionDenied => return Ok(Vec::new()),
        Err(err) => return Err(format!("failed reading {}: {err}", path.display())),
    };

    let mut items = read_dir
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
        .collect::<Vec<_>>();

    if opts.show_hidden && !opts.almost_all {
        // GNU ls -a behavior: include implied . and .. first
        let mut with_implied = vec![path.join("."), path.join("..")];
        with_implied.append(&mut items);
        Ok(with_implied)
    } else {
        Ok(items)
    }
}

fn print_help() {
    let style = Style::for_stdout();
    let theme = theme::active(None);
    let cmd = |s: &str| style.paint(theme.title, s);
    let opt = |s: &str| style.paint(theme.accent, s);
    let arg = |s: &str| style.paint(theme.ok, s);
    let desc = |s: &str| style.paint(theme.info, s);
    println!("{}", cmd("dusk ls (ls-compatible, eza-style enhancements)"));
    println!();
    println!("{}", opt("USAGE"));
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("ls"),
        arg("[OPTIONS] [FILE|DIR]...")
    );
    println!();
    println!("{}", opt("COMMON FLAGS"));
    println!(
        "  {}, {} {}",
        opt("-a"),
        opt("--all"),
        desc("Include hidden files and implied . and ..")
    );
    println!(
        "  {}, {} {}",
        opt("-A"),
        opt("--almost-all"),
        desc("Include hidden files except implied . and ..")
    );
    println!(
        "  {}, {} {}",
        opt("-l"),
        opt("--long"),
        desc("Long listing format")
    );
    println!("  {} {}", opt("-H"), desc("Print column headers"));
    println!(
        "  {}, {} {}",
        opt("-r"),
        opt("--reverse"),
        desc("Reverse sort order")
    );
    println!("  {} {}", opt("-t"), desc("Sort by modification time"));
    println!("  {} {}", opt("-S"), desc("Sort by file size"));
    println!(
        "  {}, {} {}",
        opt("-h"),
        opt("--human-readable"),
        desc("Human-readable sizes in long mode")
    );
    println!(
        "  {} {}",
        opt("--file-type"),
        desc("Append file type indicator (no executable *)")
    );
    println!(
        "  {} {}",
        opt("--author"),
        desc("With -l, print author column")
    );
    println!(
        "  {} {} {}",
        opt("--sort"),
        arg("<column>"),
        desc("name|size|time|owner|author|type|ext")
    );
    println!(
        "  {}={} {}",
        opt("--color"),
        arg("<when>"),
        desc("auto | always | never")
    );
    println!();
    println!("{}", opt("ENHANCED FLAGS"));
    println!(
        "  {} {}",
        opt("--icons"),
        desc("Enable Nerd Font icons (default)")
    );
    println!("  {} {}", opt("--no-icons"), desc("Disable icons"));
    println!(
        "  {} {}",
        opt("--basic"),
        desc("Classic plain ls output (no color, no icons)")
    );
    println!(
        "  {} {} {}",
        opt("--theme"),
        arg("<name>"),
        desc("Select color theme")
    );
    println!(
        "  {}, {} {}",
        opt("-?"),
        opt("--help"),
        desc("Show this help")
    );
}

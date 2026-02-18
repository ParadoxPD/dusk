use std::ffi::OsString;

use crate::commands;
use crate::core::icons;
use crate::core::style::Style;
use crate::core::theme;

pub fn run(argv: Vec<String>) -> Result<(), String> {
    let mut args = argv.into_iter();
    let _bin = args.next();

    let Some(cmd) = args.next() else {
        print_help();
        return Ok(());
    };

    match cmd.as_str() {
        "ls" | "eza" => {
            let ls_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::ls::run(&ls_args)
        }
        "cat" => {
            let cat_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::cat::run_cat(&cat_args)
        }
        "bat" => {
            let cat_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::cat::run_bat(&cat_args)
        }
        "find" | "rg" | "grep" => {
            let passthrough_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::passthrough::run(&cmd, &passthrough_args)
        }
        "xtree" | "tree" => {
            let xtree_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::xtree::run(&xtree_args)
        }
        "git" => {
            let git_args: Vec<String> = args.collect();
            commands::git::run(&git_args)
        }
        "diff" => {
            let diff_args: Vec<String> = args.collect();
            commands::diff::run(&diff_args)
        }
        "dump" => {
            let dump_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::dump::run(&dump_args)
        }
        "themes" => {
            let sub = args.next();
            if matches!(sub.as_deref(), None | Some("list")) {
                commands::themes::list();
                Ok(())
            } else {
                Err("themes supports only: list".to_string())
            }
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => Err(format!("unknown command: {cmd}. run `dusk help`")),
    }
}

fn print_help() {
    let style = Style::for_stdout();
    let theme = theme::active(None);
    let cmd = |s: &str| style.paint(theme.title, s);
    let opt = |s: &str| style.paint(theme.accent, s);
    let arg = |s: &str| style.paint(theme.ok, s);
    let desc = |s: &str| style.paint(theme.info, s);
    println!(
        "{}",
        cmd("dusk: one terminal tool for tree/find/grep/ls/cat + git visualization")
    );
    println!();
    println!("{} {}", opt(icons::ICON_TREE), cmd("Native commands"));
    println!(
        "  {} {} {}        {}",
        opt("dusk"),
        cmd("tree"),
        arg("[args...]"),
        desc("# Rust tree implementation (xtree engine)")
    );
    println!(
        "  {} {} {}          {}",
        opt("dusk"),
        cmd("ls"),
        arg("[args...]"),
        desc("# Rust eza-style listing")
    );
    println!(
        "  {} {} {}         {}",
        opt("dusk"),
        cmd("cat"),
        arg("[args...]"),
        desc("# Plain cat-compatible behavior")
    );
    println!(
        "  {} {} {}         {}",
        opt("dusk"),
        cmd("bat"),
        arg("[args...]"),
        desc("# Pretty themed file viewer")
    );
    println!(
        "  {} {} {}       {}",
        opt("dusk"),
        cmd("xtree"),
        arg("[args...]"),
        desc("# Extended tree/analyzer mode")
    );
    println!(
        "  {} {} {}    {}",
        opt("dusk"),
        cmd("git log"),
        arg("[theme]"),
        desc("# Informative git history graph")
    );
    println!(
        "  {} {} {}  {}",
        opt("dusk"),
        cmd("git graph"),
        arg("[theme]"),
        desc("# Alias of git log")
    );
    println!(
        "  {} {} {} {}",
        opt("dusk"),
        cmd("git status"),
        arg("[theme]"),
        desc("# Staged/modified/untracked panel")
    );
    println!(
        "  {} {} {}    {}",
        opt("dusk"),
        cmd("git tui"),
        arg("[theme]"),
        desc("# Interactive git panel (vim-style keys)")
    );
    println!(
        "  {} {} {} {}",
        opt("dusk"),
        cmd("diff"),
        arg("[theme] [--staged]"),
        desc("# Side-by-side git diff with line numbers")
    );
    println!(
        "  {} {} {}         {}",
        opt("dusk"),
        cmd("dump"),
        arg("[--hex|--asm|--both] <file>..."),
        desc("# Colorful hex + assembly dumper")
    );
    println!(
        "  {} {} {}           {}",
        opt("dusk"),
        cmd("themes"),
        arg("list"),
        desc("# Theme catalog")
    );
    println!();
    println!("{} {}", opt(icons::ICON_GIT), cmd("Pass-through"));
    println!(
        "  {} {} {}              {}",
        opt("dusk"),
        cmd("find"),
        arg("[args...]"),
        desc("# System find")
    );
    println!(
        "  {} {} {}                {}",
        opt("dusk"),
        cmd("rg"),
        arg("[args...]"),
        desc("# rg, or grep fallback if rg is missing")
    );
    println!();
    println!("{}", cmd("Quick start"));
    println!("  {} {} {}", opt("dusk"), cmd("xtree"), arg("--tldr"));
    println!("  {} {} {}", opt("dusk"), cmd("xtree"), arg("--help"));
    println!("  {} {} {}", opt("dusk"), cmd("ls"), arg("-laht"));
    println!("  {} {} {}", opt("dusk"), cmd("cat"), arg("src/main.rs"));
    println!("  {} {} {}", opt("dusk"), cmd("bat"), arg("src/main.rs"));
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("dump"),
        arg("--both target/release/dusk")
    );
    println!("  {} {} {}", opt("dusk"), cmd("git log"), arg(""));
    println!("  {} {} {}", opt("dusk"), cmd("git tui"), arg(""));
}

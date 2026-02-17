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
        "tree" => {
            let tree_args: Vec<OsString> = args.map(OsString::from).collect();
            commands::tree::run(&tree_args)
        }
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
        "xtree" => {
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
        "{}\n\n{} {}\n  {} {} {}        {}\n  {} {} {}          {}\n  {} {} {}         {}\n  {} {} {}         {}\n  {} {} {}       {}\n  {} {} {}    {}\n  {} {} {}  {}\n  {} {} {} {}\n  {} {} {} {}\n  {} {} {}           {}\n\n{} {}\n  {} {} {}              {}\n  {} {} {}                {}\n\n{}\n  {} {} {}\n  {} {} {}\n  {} {} {}\n  {} {} {}\n  {} {} {}\n  {} {} {}\n",
        cmd("dusk: one terminal tool for tree/find/grep/ls/cat + git visualization"),
        opt(icons::ICON_TREE),
        cmd("Native commands"),
        opt("dusk"),
        cmd("tree"),
        arg("[args...]"),
        desc("# Rust tree implementation (xtree engine)"),
        opt("dusk"),
        cmd("ls"),
        arg("[args...]"),
        desc("# Rust eza-style listing"),
        opt("dusk"),
        cmd("cat"),
        arg("[args...]"),
        desc("# Plain cat-compatible behavior"),
        opt("dusk"),
        cmd("bat"),
        arg("[args...]"),
        desc("# Pretty themed file viewer"),
        opt("dusk"),
        cmd("xtree"),
        arg("[args...]"),
        desc("# Extended tree/analyzer mode"),
        opt("dusk"),
        cmd("git log"),
        arg("[theme]"),
        desc("# Informative git history graph"),
        opt("dusk"),
        cmd("git graph"),
        arg("[theme]"),
        desc("# Alias of git log"),
        opt("dusk"),
        cmd("git status"),
        arg("[theme]"),
        desc("# Staged/modified/untracked panel"),
        opt("dusk"),
        cmd("diff"),
        arg("[theme] [--staged]"),
        desc("# Side-by-side git diff with line numbers"),
        opt("dusk"),
        cmd("themes"),
        arg("list"),
        desc("# Theme catalog"),
        opt(icons::ICON_GIT),
        cmd("Pass-through"),
        opt("dusk"),
        cmd("find"),
        arg("[args...]"),
        desc("# System find"),
        opt("dusk"),
        cmd("rg"),
        arg("[args...]"),
        desc("# rg, or grep fallback if rg is missing"),
        cmd("Quick start"),
        opt("dusk"),
        cmd("xtree"),
        arg("--tldr"),
        opt("dusk"),
        cmd("xtree"),
        arg("--help"),
        opt("dusk"),
        cmd("ls"),
        arg("-laht"),
        opt("dusk"),
        cmd("cat"),
        arg("src/main.rs"),
        opt("dusk"),
        cmd("bat"),
        arg("src/main.rs"),
        opt("dusk"),
        cmd("git log"),
        arg(""),
    );
}

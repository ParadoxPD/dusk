use std::ffi::OsString;

use crate::commands;
use crate::core::icons;
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
    let theme = theme::active(None);
    println!(
        "{}dusk{}: one terminal tool for tree/find/grep/ls/cat + git visualization\n\n{}{} Native commands{}\n  {} dusk tree [args...]{}          # Rust tree implementation (xtree engine)\n  {} dusk ls [args...]{}            # Rust eza-style listing\n  {} dusk cat [args...]{}           # Plain cat-compatible behavior\n  {} dusk bat [args...]{}           # Pretty themed file viewer\n  {} dusk xtree [args...]{}         # Extended tree/analyzer mode\n  {} dusk git log [theme]{}         # Informative git history graph\n  {} dusk git graph [theme]{}       # Alias of git log\n  {} dusk git status [theme]{}      # Staged/modified/untracked panel\n  {} dusk diff [theme] [--staged]{} # Side-by-side git diff with line numbers\n  {} dusk themes list{}             # Theme catalog\n\n{}{} Pass-through{}\n  {} dusk find [args...]{}          # System find\n  {} dusk rg [args...]{}            # rg, or grep fallback if rg is missing\n\n{}Quick start:{}\n  dusk xtree --tldr\n  dusk xtree --help\n  dusk ls -laht\n  dusk cat src/main.rs\n  dusk bat src/main.rs\n  dusk git log\n",
        theme.title,
        theme.reset,
        theme.accent,
        icons::ICON_TREE,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.accent,
        icons::ICON_GIT,
        theme.reset,
        theme.info,
        theme.reset,
        theme.info,
        theme.reset,
        theme.title,
        theme.reset,
    );
}

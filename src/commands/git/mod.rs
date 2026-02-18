mod tui;

use crate::core::icons;
use crate::core::process;
use crate::core::style::Style;
use crate::core::theme;

pub fn run(args: &[String]) -> Result<(), String> {
    process::ensure_command_exists("git", "dusk git")?;

    if args.iter().any(|a| a == "-h" || a == "--help") {
        let t = theme::active(None);
        let s = Style::for_stdout();
        let cmd = |x: &str| s.paint(t.title, x);
        let opt = |x: &str| s.paint(t.accent, x);
        let arg = |x: &str| s.paint(t.ok, x);
        let desc = |x: &str| s.paint(t.info, x);
        println!("{}", cmd("dusk git (enhanced git visualizer)"));
        println!();
        println!("{}", opt("USAGE"));
        println!("  {} {} {}", opt("dusk"), cmd("git log"), arg("[theme]"));
        println!("  {} {} {}", opt("dusk"), cmd("git graph"), arg("[theme]"));
        println!("  {} {} {}", opt("dusk"), cmd("git status"), arg("[theme]"));
        println!("  {} {} {}", opt("dusk"), cmd("git viz"), arg("[theme]"));
        println!("  {} {} {}", opt("dusk"), cmd("git tui"), arg("[theme]"));
        println!();
        println!("{}", opt("DESCRIPTION"));
        println!(
            "  {} {}",
            cmd("git log/graph"),
            desc("Informative commit graph (VSCode-style)")
        );
        println!(
            "  {} {}",
            cmd("git status/viz"),
            desc("Staged/modified/untracked panel")
        );
        println!(
            "  {} {}",
            cmd("git tui"),
            desc("Interactive git panel with vim-style navigation")
        );
        println!();
        println!("{}", opt("TUI KEYS"));
        println!(
            "  {}",
            desc(
                "1/2/3 tabs  j/k move  h/l pane  s/u stage/unstage  A/U all  c commit  p push current  R push-remote  b/B branch  t cycle theme  Ctrl+P palette  : command (use :cmdhelp)  ? help  q quit"
            )
        );
        return Ok(());
    }

    let sub = args.first().map(|s| s.as_str()).unwrap_or("graph");

    match sub {
        "graph" | "log" => log_graph(args.get(1).map(String::as_str)),
        "status" | "viz" => status_panel(args.get(1).map(String::as_str)),
        "tui" | "interactive" => tui::run(args.get(1).map(String::as_str)),
        _ => Err("git supports: graph | log | status | tui".to_string()),
    }
}

fn log_graph(theme_name: Option<&str>) -> Result<(), String> {
    let theme = theme::active(theme_name);
    let style = Style::for_stdout();

    println!(
        "{}",
        style.paint(
            theme.title,
            format!("{} Git History Graph", icons::ICON_GIT)
        )
    );

    let color_mode = if style.color {
        "--color=always"
    } else {
        "--color=never"
    };

    let output = process::run_capture(
        "git",
        &[
            "log",
            "--graph",
            "--all",
            "--decorate",
            "--date=relative",
            "--pretty=format:%C(bold blue)%h%Creset%x09%C(yellow)%d%Creset%x09%s%x09%C(cyan)%an%Creset%x09%C(green)%cr%Creset",
            color_mode,
        ],
    )
    .map_err(|err| format!("failed to run git log graph: {err}"))?;

    print!("{output}");
    println!();
    Ok(())
}

fn status_panel(theme_name: Option<&str>) -> Result<(), String> {
    let theme = theme::active(theme_name);
    let style = Style::for_stdout();
    let branch = process::run_capture("git", &["branch", "--show-current"])
        .map_err(|err| format!("failed to read branch: {err}"))?;
    let porcelain = process::run_capture("git", &["status", "--porcelain"])
        .map_err(|err| format!("failed to read status: {err}"))?;

    println!(
        "{}",
        style.paint(
            theme.accent,
            format!("{} Branch: {}", icons::ICON_BRANCH, branch.trim())
        )
    );

    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut untracked = Vec::new();

    for line in porcelain.lines() {
        if line.len() < 4 {
            continue;
        }
        let x = line.chars().next().unwrap_or(' ');
        let y = line.chars().nth(1).unwrap_or(' ');
        let path = line[3..].to_string();

        if x == '?' {
            untracked.push(path);
            continue;
        }

        if x != ' ' && x != '?' {
            staged.push(path.clone());
        }
        if y != ' ' {
            modified.push(path.clone());
        }
    }

    println!(
        "{}",
        style.paint(theme.ok, format!("{} Staged", icons::ICON_STAGED))
    );
    if staged.is_empty() {
        println!("  {}", style.paint(theme.info, "none"));
    } else {
        for path in staged {
            println!("  {}", style.paint(theme.info, path));
        }
    }

    println!(
        "{}",
        style.paint(theme.warn, format!("{} Modified", icons::ICON_MODIFIED))
    );
    if modified.is_empty() {
        println!("  {}", style.paint(theme.info, "none"));
    } else {
        for path in modified {
            println!("  {}", style.paint(theme.info, path));
        }
    }

    println!(
        "{}",
        style.paint(theme.accent, format!("{} Untracked", icons::ICON_UNTRACKED))
    );
    if untracked.is_empty() {
        println!("  {}", style.paint(theme.info, "none"));
    } else {
        for path in untracked {
            println!("  {}", style.paint(theme.info, path));
        }
    }

    Ok(())
}

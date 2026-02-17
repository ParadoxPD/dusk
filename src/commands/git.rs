use crate::core::icons;
use crate::core::process;
use crate::core::theme;

pub fn run(args: &[String]) -> Result<(), String> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("graph");

    match sub {
        "graph" => graph(args.get(1).map(String::as_str)),
        "status" | "viz" => status_panel(args.get(1).map(String::as_str)),
        _ => Err("git supports: graph | status".to_string()),
    }
}

fn graph(theme_name: Option<&str>) -> Result<(), String> {
    let theme = theme::active(theme_name);
    let color_mode = if theme.reset.is_empty() {
        "--color=never"
    } else {
        "--color=always"
    };
    println!(
        "{}{} Git Graph{}",
        theme.accent,
        icons::ICON_GIT,
        theme.reset
    );

    let output = process::run_capture(
        "git",
        &[
            "log",
            "--graph",
            "--decorate",
            "--oneline",
            "--all",
            color_mode,
        ],
    )
    .map_err(|err| format!("failed to run git log graph: {err}"))?;

    print!("{output}");
    Ok(())
}

fn status_panel(theme_name: Option<&str>) -> Result<(), String> {
    let theme = theme::active(theme_name);
    let branch = process::run_capture("git", &["branch", "--show-current"])
        .map_err(|err| format!("failed to read branch: {err}"))?;
    let porcelain = process::run_capture("git", &["status", "--porcelain"])
        .map_err(|err| format!("failed to read status: {err}"))?;

    println!(
        "{}{} {} {}{}",
        theme.accent,
        icons::ICON_BRANCH,
        "Branch:",
        branch.trim(),
        theme.reset
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

    println!("{}{} Staged{}", theme.ok, icons::ICON_STAGED, theme.reset);
    if staged.is_empty() {
        println!("  {}none{}", theme.subtle, theme.reset);
    } else {
        for path in staged {
            println!("  {path}");
        }
    }

    println!(
        "{}{} Modified{}",
        theme.warn,
        icons::ICON_MODIFIED,
        theme.reset
    );
    if modified.is_empty() {
        println!("  {}none{}", theme.subtle, theme.reset);
    } else {
        for path in modified {
            println!("  {path}");
        }
    }

    println!(
        "{}{} Untracked{}",
        theme.accent,
        icons::ICON_UNTRACKED,
        theme.reset
    );
    if untracked.is_empty() {
        println!("  {}none{}", theme.subtle, theme.reset);
    } else {
        for path in untracked {
            println!("  {path}");
        }
    }

    Ok(())
}

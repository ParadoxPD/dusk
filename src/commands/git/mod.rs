mod diff_tui;
mod diffview;
mod tui;

use crate::core::icons;
use crate::core::process;
use crate::core::style::Style;
use crate::core::theme;

pub fn run(args: &[String]) -> Result<(), String> {
    process::ensure_command_exists("git", "dusk git")?;

    if args.is_empty() || matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
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
        println!("  {} {} {}", opt("dusk"), cmd("git status"), arg("[theme]"));
        println!(
            "  {} {} {}",
            opt("dusk"),
            cmd("git diff"),
            arg("[theme] [--staged] [--tui]")
        );
        println!("  {} {} {}", opt("dusk"), cmd("git tui"), arg("[theme]"));
        println!();
        println!("{}", opt("DESCRIPTION"));
        println!(
            "  {} {}",
            cmd("git log"),
            desc("Informative commit graph (VSCode-style)")
        );
        println!(
            "  {} {}",
            cmd("git status"),
            desc("Staged/modified/untracked panel")
        );
        println!(
            "  {} {}",
            cmd("git diff"),
            desc("Side-by-side line-numbered diff with syntax highlighting")
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

    let sub = args.first().map(|s| s.as_str()).unwrap_or("log");

    match sub {
        "log" => log_graph(args.get(1).map(String::as_str)),
        "status" => status_panel(args.get(1).map(String::as_str)),
        "diff" => git_diff(&args[1..]),
        "tui" | "interactive" => tui::run(args.get(1).map(String::as_str)),
        "graph" => {
            Err("`dusk git graph` was removed as redundant. Use `dusk git log`.".to_string())
        }
        "viz" => Err("`dusk git viz` was removed as redundant. Use `dusk git status`.".to_string()),
        _ => Err("git supports: log | status | diff | tui".to_string()),
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

    let output = process::run_capture(
        "git",
        &[
            "log",
            "--graph",
            "--all",
            "--decorate",
            "--date=relative",
            "--color=never",
            "--pretty=format:%h%x1f%d%x1f%s%x1f%an%x1f%cr",
        ],
    )
    .map_err(|err| format!("failed to run git log graph: {err}"))?;

    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v >= 80)
        .unwrap_or(140);
    for line in format_graph_lines(&output, &style, theme, width) {
        println!("{line}");
    }
    println!();
    Ok(())
}

fn git_diff(args: &[String]) -> Result<(), String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        let t = theme::active(None);
        let s = Style::for_stdout();
        let cmd = |x: &str| s.paint(t.title, x);
        let opt = |x: &str| s.paint(t.accent, x);
        let arg = |x: &str| s.paint(t.ok, x);
        let desc = |x: &str| s.paint(t.info, x);
        println!("{}", cmd("dusk git diff"));
        println!();
        println!("{}", opt("USAGE"));
        println!(
            "  {} {} {}",
            opt("dusk"),
            cmd("git diff"),
            arg("[theme] [--staged] [--tui]")
        );
        println!();
        println!("{}", opt("FLAGS"));
        println!("  {} {}", opt("--staged"), desc("Show staged changes"));
        println!("  {} {}", opt("--tui"), desc("Open isolated diff TUI"));
        println!("  {} {}", opt("--no-tui"), desc("Force terminal output"));
        return Ok(());
    }

    let mut theme_name: Option<&str> = None;
    let mut staged = false;
    let mut use_tui = false;
    for arg in args {
        match arg.as_str() {
            "--staged" => staged = true,
            "--tui" => use_tui = true,
            "--no-tui" => use_tui = false,
            s if !s.starts_with('-') => theme_name = Some(s),
            _ => {}
        }
    }

    if use_tui {
        return diff_tui::run(theme_name, staged);
    }

    let theme = theme::active(theme_name);
    let style = Style::for_stdout();
    let mut diff_args = vec!["diff", "--no-color", "--unified=3"];
    if staged {
        diff_args.push("--staged");
    }
    let output = process::run_capture("git", &diff_args)
        .map_err(|e| format!("failed to run git diff: {e}"))?;

    if output.trim().is_empty() {
        println!("{}", style.paint(theme.info, "No changes to diff."));
        return Ok(());
    }

    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v >= 80)
        .unwrap_or(140);
    for line in diffview::render_side_by_side(&output, &style, theme, width) {
        println!("{line}");
    }
    Ok(())
}

fn format_graph_lines(
    output: &str,
    style: &Style,
    theme: theme::Theme,
    width: usize,
) -> Vec<String> {
    #[derive(Clone)]
    enum Row {
        Link(String),
        Commit {
            graph_hash: String,
            deco: String,
            subject: String,
            author: String,
            age: String,
        },
    }

    let mut rows = Vec::new();
    for line in output.lines() {
        if !line.contains('\u{1f}') {
            rows.push(Row::Link(line.to_string()));
            continue;
        }
        let cols = line.split('\u{1f}').collect::<Vec<_>>();
        let first = cols.first().copied().unwrap_or_default();
        let (graph, hash) = split_graph_hash(first);
        rows.push(Row::Commit {
            graph_hash: format!("{graph}{hash}"),
            deco: cols.get(1).copied().unwrap_or_default().to_string(),
            subject: cols.get(2).copied().unwrap_or_default().to_string(),
            author: cols.get(3).copied().unwrap_or_default().to_string(),
            age: cols.get(4).copied().unwrap_or_default().to_string(),
        });
    }

    let mut w_graph = 0usize;
    let mut w_deco = 0usize;
    let mut w_author = 0usize;
    let mut w_age = 0usize;
    for row in &rows {
        if let Row::Commit {
            graph_hash,
            deco,
            author,
            age,
            ..
        } = row
        {
            w_graph = w_graph.max(graph_hash.chars().count());
            w_deco = w_deco.max(deco.chars().count());
            w_author = w_author.max(author.chars().count());
            w_age = w_age.max(age.chars().count());
        }
    }
    w_graph = w_graph.clamp(10, 24);
    w_deco = w_deco.clamp(0, 28);
    w_author = w_author.clamp(8, 20);
    w_age = w_age.clamp(8, 16);
    let sep_w = 2 * 4; // four "  " separators
    let fixed = w_graph + w_deco + w_author + w_age + sep_w;
    let subject_w = width.saturating_sub(fixed).max(16);

    let mut out = Vec::new();
    for row in rows {
        match row {
            Row::Link(line) => out.push(style.paint(theme.subtle, line)),
            Row::Commit {
                graph_hash,
                deco,
                subject,
                author,
                age,
            } => {
                let gh = pad_text(&graph_hash, w_graph);
                let d = pad_text(&deco, w_deco);
                let s = pad_text(&subject, subject_w);
                let a = pad_text(&author, w_author);
                let age = pad_text(&age, w_age);
                out.push(format!(
                    "{}  {}  {}  {}  {}",
                    style.paint(theme.accent, gh),
                    style.paint(theme.warn, d),
                    style.paint(theme.info, s),
                    style.paint(theme.ok, a),
                    style.paint(theme.number, age),
                ));
            }
        }
    }
    out
}

fn split_graph_hash(s: &str) -> (&str, &str) {
    for tok in s.split_whitespace() {
        if tok.len() >= 7 && tok.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Some(idx) = s.find(tok) {
                return (&s[..idx], tok);
            }
        }
    }
    (s, "")
}

fn pad_text(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let len = s.chars().count();
    if len > width {
        if width <= 1 {
            return "…".to_string();
        }
        let mut out = String::new();
        for (i, ch) in s.chars().enumerate() {
            if i + 1 >= width {
                out.push('…');
                break;
            }
            out.push(ch);
        }
        out
    } else if len < width {
        format!("{s:<width$}")
    } else {
        s.to_string()
    }
}

fn status_panel(theme_name: Option<&str>) -> Result<(), String> {
    let theme = theme::active(theme_name);
    let style = Style::for_stdout();
    let branch = process::run_capture("git", &["branch", "--show-current"])
        .map_err(|err| format!("failed to read branch: {err}"))?;
    let branch = branch.trim().to_string();
    let upstream = process::run_capture(
        "git",
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());

    let branch_relation = if let Some(up) = &upstream {
        match process::run_capture(
            "git",
            &[
                "rev-list",
                "--left-right",
                "--count",
                &format!("{up}...HEAD"),
            ],
        ) {
            Ok(counts) => {
                let parts = counts.split_whitespace().collect::<Vec<_>>();
                if parts.len() == 2 {
                    let behind = parts[0].parse::<usize>().unwrap_or(0);
                    let ahead = parts[1].parse::<usize>().unwrap_or(0);
                    if ahead == 0 && behind == 0 {
                        format!("Your branch is up to date with '{}'.", up)
                    } else if ahead > 0 && behind == 0 {
                        format!("Your branch is ahead of '{}' by {} commit(s).", up, ahead)
                    } else if ahead == 0 && behind > 0 {
                        format!("Your branch is behind '{}' by {} commit(s).", up, behind)
                    } else {
                        format!(
                            "Your branch and '{}' have diverged (ahead {}, behind {}).",
                            up, ahead, behind
                        )
                    }
                } else {
                    format!("Tracking '{}'.", up)
                }
            }
            Err(_) => format!("Tracking '{}'.", up),
        }
    } else {
        "Your branch has no upstream branch.".to_string()
    };

    let porcelain = process::run_capture("git", &["status", "--porcelain"])
        .map_err(|err| format!("failed to read status: {err}"))?;

    println!(
        "{}",
        style.paint(
            theme.accent,
            format!("{} On branch {}", icons::ICON_BRANCH, branch)
        )
    );
    println!("{}", style.paint(theme.info, branch_relation));
    println!();

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

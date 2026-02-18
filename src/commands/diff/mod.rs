use std::env;

use crate::core::icons;
use crate::core::process;
use crate::core::style::Style;
use crate::core::theme;

pub fn run(args: &[String]) -> Result<(), String> {
    process::ensure_command_exists("git", "dusk diff")?;

    if args.iter().any(|a| a == "-h" || a == "--help") {
        let theme = theme::active(None);
        let style = Style::for_stdout();
        let cmd = |s: &str| style.paint(theme.title, s);
        let opt = |s: &str| style.paint(theme.accent, s);
        let arg = |s: &str| style.paint(theme.ok, s);
        let desc = |s: &str| style.paint(theme.info, s);
        println!(
            "{}",
            cmd("dusk diff (side-by-side git diff with line numbers)")
        );
        println!();
        println!("{}", opt("USAGE"));
        println!(
            "  {} {} {}",
            opt("dusk"),
            cmd("diff"),
            arg("[theme] [--staged]")
        );
        println!();
        println!("{}", opt("FLAGS"));
        println!("  {} {}", opt("--staged"), desc("Show staged changes"));
        println!(
            "  {}, {} {}",
            opt("-h"),
            opt("--help"),
            desc("Show this help")
        );
        return Ok(());
    }

    let mut theme_name: Option<&str> = None;
    let mut staged = false;

    for arg in args {
        match arg.as_str() {
            "--staged" => staged = true,
            other if !other.starts_with('-') => theme_name = Some(other),
            _ => {}
        }
    }

    let theme = theme::active(theme_name);
    let style = Style::for_stdout();

    println!(
        "{}",
        style.paint(
            theme.title,
            format!("{} Side-by-Side Diff", icons::ICON_DIFF)
        )
    );

    let mut diff_args = vec!["diff", "--no-color", "--unified=3"];
    if staged {
        diff_args.push("--staged");
    }

    let output = process::run_capture("git", &diff_args)
        .map_err(|err| format!("failed to run git diff: {err}"))?;

    if output.trim().is_empty() {
        println!("{}", style.paint(theme.info, "No changes to diff."));
        return Ok(());
    }

    render_diff(&output, &style, theme);
    Ok(())
}

fn render_diff(output: &str, style: &Style, theme: theme::Theme) {
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 60)
        .unwrap_or(140);

    let num_w = 6;
    let sep = " │ ";
    let content_w = width.saturating_sub(num_w * 2 + sep.len() + 7) / 2;

    let lines = output.lines().collect::<Vec<_>>();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(rest) = line.strip_prefix("diff --git ") {
            println!();
            println!("{}", style.paint(theme.accent, format!("{rest}")));
            i += 1;
            continue;
        }

        if line.starts_with("@@ ") {
            let (mut old_ln, mut new_ln) = parse_hunk_header(line);
            println!("{}", style.paint(theme.number, line));
            i += 1;

            while i < lines.len() {
                let cur = lines[i];
                if cur.starts_with("diff --git ") || cur.starts_with("@@ ") {
                    break;
                }
                if cur.starts_with("index ")
                    || cur.starts_with("--- ")
                    || cur.starts_with("+++ ")
                    || cur.starts_with("new file mode")
                    || cur.starts_with("deleted file mode")
                    || cur.starts_with("similarity index")
                    || cur.starts_with("rename from ")
                    || cur.starts_with("rename to ")
                {
                    i += 1;
                    continue;
                }

                if cur.starts_with('-') && i + 1 < lines.len() && lines[i + 1].starts_with('+') {
                    let left = &cur[1..];
                    let right = &lines[i + 1][1..];
                    print_side(
                        style,
                        theme,
                        Some(old_ln),
                        Some(new_ln),
                        left,
                        right,
                        content_w,
                        theme.warn,
                        theme.ok,
                    );
                    old_ln += 1;
                    new_ln += 1;
                    i += 2;
                    continue;
                }

                if let Some(left) = cur.strip_prefix('-') {
                    print_side(
                        style,
                        theme,
                        Some(old_ln),
                        None,
                        left,
                        "",
                        content_w,
                        theme.warn,
                        theme.info,
                    );
                    old_ln += 1;
                    i += 1;
                    continue;
                }

                if let Some(right) = cur.strip_prefix('+') {
                    print_side(
                        style,
                        theme,
                        None,
                        Some(new_ln),
                        "",
                        right,
                        content_w,
                        theme.info,
                        theme.ok,
                    );
                    new_ln += 1;
                    i += 1;
                    continue;
                }

                if let Some(ctx) = cur.strip_prefix(' ') {
                    print_side(
                        style,
                        theme,
                        Some(old_ln),
                        Some(new_ln),
                        ctx,
                        ctx,
                        content_w,
                        theme.info,
                        theme.info,
                    );
                    old_ln += 1;
                    new_ln += 1;
                    i += 1;
                    continue;
                }

                i += 1;
            }
            continue;
        }

        i += 1;
    }
}

fn parse_hunk_header(line: &str) -> (usize, usize) {
    let mut old_ln = 1usize;
    let mut new_ln = 1usize;

    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() >= 3 {
        if let Some(old) = parts[1].strip_prefix('-') {
            old_ln = old
                .split(',')
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(1);
        }
        if let Some(new) = parts[2].strip_prefix('+') {
            new_ln = new
                .split(',')
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .unwrap_or(1);
        }
    }

    (old_ln, new_ln)
}

#[allow(clippy::too_many_arguments)]
fn print_side(
    style: &Style,
    theme: theme::Theme,
    old_ln: Option<usize>,
    new_ln: Option<usize>,
    left: &str,
    right: &str,
    width: usize,
    left_color: &str,
    right_color: &str,
) {
    let left_no = old_ln
        .map(|n| format!("{n:>6}"))
        .unwrap_or_else(|| "      ".to_string());
    let right_no = new_ln
        .map(|n| format!("{n:>6}"))
        .unwrap_or_else(|| "      ".to_string());

    let left_txt = truncate(left, width);
    let right_txt = truncate(right, width);

    let left_block = format!(
        "{} {}",
        style.paint(theme.number, left_no),
        style.paint(left_color, format!("{left_txt:<width$}"))
    );
    let right_block = format!(
        "{} {}",
        style.paint(theme.number, right_no),
        style.paint(right_color, format!("{right_txt:<width$}"))
    );

    println!(
        "{left_block} {} {right_block}",
        style.paint(theme.accent, "│")
    );
}

fn truncate(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for ch in s.chars() {
        if count >= width.saturating_sub(1) {
            out.push('…');
            return out;
        }
        out.push(ch);
        count += 1;
    }
    out
}

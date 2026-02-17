use crate::core::icons;
use crate::core::process;
use crate::core::theme;

pub fn run(args: &[String]) -> Result<(), String> {
    let theme_name = args.first().map(String::as_str);
    let theme = theme::active(theme_name);

    println!(
        "{}{} Beautiful Diff{}",
        theme.accent,
        icons::ICON_DIFF,
        theme.reset
    );

    let mut diff_args = vec![
        "diff",
        if theme.reset.is_empty() {
            "--color=never"
        } else {
            "--color=always"
        },
        "--word-diff=color",
    ];
    if args.iter().any(|arg| arg == "--staged") {
        diff_args.push("--staged");
    }

    let output = process::run_capture("git", &diff_args)
        .map_err(|err| format!("failed to run git diff: {err}"))?;

    if output.trim().is_empty() {
        println!("{}No changes to diff.{}", theme.subtle, theme.reset);
        return Ok(());
    }

    print!("{output}");
    Ok(())
}

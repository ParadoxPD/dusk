use crate::core::style::Style;
use crate::core::theme;

pub fn print_help() {
    let style = Style::for_stdout();
    let t = theme::active(None);
    let cmd = |s: &str| style.paint(t.title, s);
    let opt = |s: &str| style.paint(t.accent, s);
    let arg = |s: &str| style.paint(t.ok, s);
    let desc = |s: &str| style.paint(t.info, s);

    println!("{}", cmd("dusk rm (safe rm with trash + scanner TUI)"));
    println!();
    println!("{}", opt("USAGE"));
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("rm"),
        arg("[OPTIONS] FILE...")
    );
    println!("  {} {}", opt("dusk"), cmd("rm --trash-tui"));
    println!();

    println!("{}", opt("DROP-IN FLAGS"));
    println!(
        "  {}, {} {}",
        opt("-f"),
        opt("--force"),
        desc("Ignore missing paths, never prompt")
    );
    println!(
        "  {}, {} {}",
        opt("-i"),
        opt("--interactive"),
        desc("Prompt before every removal")
    );
    println!(
        "  {}, {}, {} {}",
        opt("-r"),
        opt("-R"),
        opt("--recursive"),
        desc("Remove directories and their contents")
    );
    println!(
        "  {}, {} {}",
        opt("-v"),
        opt("--verbose"),
        desc("Print action for each processed path")
    );
    println!();

    println!("{}", opt("SAFE DELETE"));
    println!(
        "  {}, {} {}",
        opt("-P"),
        opt("--permanent"),
        desc("Hard delete (skip trash)")
    );
    println!(
        "  {} {}",
        opt("--trash"),
        desc("Force soft-delete mode (default)")
    );
    println!(
        "  {}, {} {}",
        opt("--trash-tui"),
        opt("--scan-trash"),
        desc("Open interactive trash scanner")
    );
    println!(
        "  {} {} {}",
        opt("--restore"),
        arg("<id|pattern>"),
        desc("Restore matching entries from trash")
    );
    println!(
        "  {} {}",
        opt("--empty-trash"),
        desc("Permanently delete all trash entries")
    );
    println!();

    println!("{}", opt("TUI KEYS"));
    println!("  {}", desc("j/k or Up/Down move selection"));
    println!("  {}", desc("Space mark/unmark"));
    println!("  {}", desc("r restore selected/marked entries"));
    println!("  {}", desc("d permanently delete selected/marked entries"));
    println!("  {}", desc("g/G top/bottom"));
    println!("  {}", desc("q/Esc quit"));
    println!();

    println!("{}", opt("TLDR"));
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("rm"),
        arg("file.txt      # move to trash")
    );
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("rm -r"),
        arg("build/        # move dir to trash")
    );
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("rm -rP"),
        arg("build/        # hard delete")
    );
    println!(
        "  {} {} {}",
        opt("dusk"),
        cmd("rm --restore"),
        arg("important.txt")
    );
    println!("  {} {}", opt("dusk"), cmd("rm --empty-trash"));
    println!("  {}", desc("DUSK_TRASH_DIR can override trash location"));
}

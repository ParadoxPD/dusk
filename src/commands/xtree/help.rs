use crate::core::style::Style;
use crate::core::theme;

pub fn tldr_help(theme_name: Option<&str>) -> String {
    let style = Style::for_stdout();
    let t = theme::active(theme_name);
    let cmd = |s: &str| style.paint(t.title, s);
    let opt = |s: &str| style.paint(t.accent, s);
    let arg = |s: &str| style.paint(t.ok, s);
    let desc = |s: &str| style.paint(t.info, s);

    let mut out = String::new();
    out.push_str(&format!("{}\n\n", cmd("dusk xtree tldr")));
    out.push_str(&format!("{}\n", opt("Main usage:")));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        desc("# tree with gitignore/.treeignore support")
    ));
    out.push_str(&format!(
        "  {} {} {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("-L 2"),
        arg("--theme nord"),
        desc("# depth-limited themed tree")
    ));
    out.push_str(&format!(
        "  {} {} {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("-g TODO"),
        arg("-c rs ts"),
        desc("# grep + inline content preview")
    ));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--stats --loc --big --dupes")
    ));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--fingerprint")
    ));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--json > tree.json")
    ));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--md > tree.md")
    ));
    out.push_str(&format!(
        "  {} {} {}\n\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--prompt > prompt.txt")
    ));
    out.push_str(&format!("{}\n", opt("Show full help:")));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--help")
    ));
    out
}

pub fn full_help(theme_name: Option<&str>) -> String {
    let style = Style::for_stdout();
    let t = theme::active(theme_name);
    let cmd = |s: &str| style.paint(t.title, s);
    let opt = |s: &str| style.paint(t.accent, s);
    let arg = |s: &str| style.paint(t.ok, s);
    let desc = |s: &str| style.paint(t.info, s);

    let mut out = String::new();
    out.push_str(&format!("{}\n\n", cmd("Enhanced Tree (pure Rust)")));
    out.push_str(&format!("{}\n", opt("USAGE")));
    out.push_str(&format!(
        "  {} {} {}\n\n",
        opt("dusk"),
        cmd("xtree"),
        arg("[OPTIONS] [DIRECTORY]")
    ));

    out.push_str(&format!("{}\n", opt("TLDR")));
    out.push_str(&format!(
        "  {} {} {}\n\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--tldr")
    ));

    out.push_str(&format!("{}\n", opt("NAVIGATION")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("-L <depth>"),
        desc("Limit recursion depth")
    ));
    out.push_str(&format!("  {} {}\n", opt("-d"), desc("Directories only")));
    out.push_str(&format!("  {} {}\n", opt("-a"), desc("Show hidden files")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("-e, --exclude <pattern>"),
        desc("Exclude pattern (repeatable)")
    ));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("-I <pattern>"),
        desc("tree-compatible alias for exclude")
    ));

    out.push_str(&format!("{}\n", opt("DISPLAY")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("-i"),
        desc("Show metadata (permissions, owner, modified)")
    ));
    out.push_str(&format!("  {} {}\n", opt("-s"), desc("Hide file sizes")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--no-icon"),
        desc("Disable Nerd Font icons")
    ));
    out.push_str(&format!("  {} {}\n", opt("--theme <name>"), desc("Theme: default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark-pro | monokai | kanagawa | everforest | rose-pine | ayu | nightfox")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--tests"),
        desc("Highlight test files")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--count"),
        desc("Show file count per directory")
    ));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("--noreport"),
        desc("Suppress final directory/file summary line")
    ));

    out.push_str(&format!("{}\n", opt("INSPECT FILES")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("-c, --cat <ext...>"),
        desc("Print file contents for extensions")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("-g, --grep <pattern>"),
        desc("Search inside files (first 5 matches per file)")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--clip <n>"),
        desc("Limit printed lines per file (default 100)")
    ));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("--no-clip, --nc"),
        desc("Disable line clipping")
    ));

    out.push_str(&format!("{}\n", opt("GIT/FILTERING")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--no-git"),
        desc("Disable .gitignore filtering")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--no-treeignore"),
        desc("Disable .treeignore filtering")
    ));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("--focus <ext...>"),
        desc("Keep only directories containing matching extensions")
    ));

    out.push_str(&format!("{}\n", opt("ANALYSIS")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--stats"),
        desc("Language/extension statistics")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--loc"),
        desc("Show total lines of code (LOC)")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--big"),
        desc("Mark files larger than 5 MB")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--dupes"),
        desc("Detect duplicate files by content hash")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--audit"),
        desc("Security audit (world-writable, suspicious executables, secret hints)")
    ));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("--fingerprint"),
        desc("Project summary (counts, size, depth, git snapshot, largest files)")
    ));

    out.push_str(&format!("{}\n", opt("ORGANIZATION")));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--sort <mode>"),
        desc("name | size | time")
    ));
    out.push_str(&format!(
        "  {} {}\n",
        opt("--group"),
        desc("Group files by extension")
    ));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("--resolve"),
        desc("Resolve symlink targets")
    ));

    out.push_str(&format!("{}\n", opt("OUTPUT FORMATS")));
    out.push_str(&format!("  {} {}\n", opt("--md"), desc("Markdown export")));
    out.push_str(&format!("  {} {}\n", opt("--json"), desc("JSON output")));
    out.push_str(&format!(
        "  {} {}\n\n",
        opt("--prompt"),
        desc("AI-friendly dump and save to your OS temp directory")
    ));

    out.push_str(&format!("{}\n", opt("EXAMPLES")));
    out.push_str(&format!("  {} {}\n", opt("dusk"), cmd("xtree")));
    out.push_str(&format!(
        "  {} {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("-L 2"),
        arg("--theme nord")
    ));
    out.push_str(&format!(
        "  {} {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--no-git"),
        arg("--sort size")
    ));
    out.push_str(&format!(
        "  {} {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("-g \"TODO\""),
        arg("-c rs toml")
    ));
    out.push_str(&format!(
        "  {} {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--focus rs ts"),
        arg("--count")
    ));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--stats --loc --big --dupes")
    ));
    out.push_str(&format!(
        "  {} {} {}\n",
        opt("dusk"),
        cmd("xtree"),
        arg("--fingerprint --dupes")
    ));
    out
}

pub fn tldr_help() -> String {
    let msg = r#"dusk xtree tldr

Main usage:
  dusk xtree                          # tree with gitignore/.treeignore support
  dusk xtree -L 2 --theme nord        # depth-limited themed tree
  dusk xtree -g TODO -c rs ts         # grep + inline content preview
  dusk xtree --stats --big --dupes    # analysis mode
  dusk xtree --fingerprint            # project summary + git snapshot
  dusk xtree --json > tree.json       # machine-readable export
  dusk xtree --md > tree.md           # markdown export
  dusk xtree --prompt > prompt.txt    # AI-ready prompt dump

Show full help:
  dusk xtree --help
"#;
    msg.to_string()
}

pub fn full_help() -> String {
    let msg = r#"Enhanced Tree (pure Rust)

USAGE
  dusk xtree [OPTIONS] [DIRECTORY]

TLDR
  dusk xtree --tldr

NAVIGATION
  -L <depth>                Limit recursion depth
  -d                        Directories only
  -a                        Show hidden files
  -e, --exclude <pattern>   Exclude pattern (repeatable)
  -I <pattern>              tree-compatible alias for exclude

DISPLAY
  -i                        Show metadata (permissions, owner, modified)
  -s                        Hide file sizes
  --no-icon                 Disable Nerd Font icons
  --theme <name>            Theme: default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark | monokai | kanagawa | everforest | rose-pine | ayu | nightfox
  --tests                   Highlight test files
  --count                   Show file count per directory
  --noreport                Suppress final directory/file summary line

INSPECT FILES
  -c, --cat <ext...>        Print file contents for extensions
  -g, --grep <pattern>      Search inside files (first 5 matches per file)
  --clip <n>                Limit printed lines per file (default 100)
  --no-clip, --nc           Disable line clipping

GIT/FILTERING
  --no-git                  Disable .gitignore filtering
  --no-treeignore           Disable .treeignore filtering
  --focus <ext...>          Keep only directories containing matching extensions

ANALYSIS
  --stats                   Language/extension statistics
  --big                     Mark files larger than 5 MB
  --dupes                   Detect duplicate files by content hash
  --audit                   Security audit (world-writable, suspicious executables, secret hints)
  --fingerprint             Project summary (counts, size, depth, git snapshot, largest files)

ORGANIZATION
  --sort <mode>             name | size | time
  --group                   Group files by extension
  --resolve                 Resolve symlink targets

OUTPUT FORMATS
  --md                      Markdown export
  --json                    JSON output
  --prompt                  AI-friendly dump and save to your OS temp directory

EXAMPLES
  dusk xtree
  dusk xtree -L 2 --theme nord
  dusk xtree --no-git --sort size
  dusk xtree -g "TODO" -c rs toml
  dusk xtree --focus rs ts --count
  dusk xtree --stats --big --dupes
  dusk xtree --fingerprint --dupes
"#;
    msg.to_string()
}

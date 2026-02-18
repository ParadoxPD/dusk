# Dusk Features

`dusk` is a unified CLI with native Rust command implementations plus focused passthrough wrappers.

## Command Overview

- `dusk help`
- `dusk xtree [OPTIONS] [DIRECTORY]`
- `dusk tree [OPTIONS] [DIRECTORY]`
- `dusk ls [OPTIONS] [DIRECTORY]`
- `dusk cat [OPTIONS] [FILE]...`
- `dusk bat [OPTIONS] [FILE]...`
- `dusk git log [theme]`
- `dusk git graph [theme]`
- `dusk git status|viz [theme]`
- `dusk git tui|interactive [theme]`
- `dusk diff [theme] [--staged]`
- `dusk dump [--hex|--asm|--both] [--theme <name>] <file>...`
- `dusk themes list`
- `dusk find [args...]`
- `dusk rg [args...]`

## xtree (Native Rust)

Use `xtree` for enhanced tree + inspection + analysis.

Help:
- `dusk xtree --help`
- `dusk xtree --tldr`

Navigation:
- `-L <depth>`: limit recursion depth
- `-d`: directories only
- `-a`: include hidden files/directories
- `-e, --exclude <pattern>`: exclusion pattern (repeatable)
- `-I <pattern>`: tree-compatible alias for exclude

Display:
- `-i`: metadata (permissions + modified time)
- `-s`: hide file sizes
- `--no-icon`: disable Nerd Font icons
- `--theme <name>`
- `--tests`: highlight test files/directories
- `--count`: per-directory file counts
- `--noreport`: hide final summary line

Inspect files:
- `-c, --cat <ext...>`: print matching file contents
- `-g, --grep <pattern>`: print first 5 matches per file
- `--clip <n>`: limit printed lines (default `100`)
- `--no-clip`, `--nc`: disable clipping

Filtering:
- `--no-git`: disable `.gitignore`
- `--no-treeignore`: disable `.treeignore`
- `--focus <ext...>`: only keep directories/files with matching extensions

Analysis:
- `--stats`: extension/language counts
- `--big`: mark files larger than 5 MB
- `--dupes`: duplicate detection by content hash
- `--audit`: permission/executable/secret checks
- `--fingerprint`: project summary report

Organization:
- `--sort <mode>`: `name | size | time`
- `--group`: grouped by extension
- `--resolve`: resolve symlink targets

Output:
- `--md`: Markdown export
- `--json`: JSON export
- `--prompt`: AI-friendly dump to temp file

## tree Subcommand (Native Rust)

- `dusk tree [OPTIONS] [DIRECTORY]`
- Uses the same native engine as `xtree`.

## ls Subcommand (Native Rust, eza-style)

Usage:
- `dusk ls [OPTIONS] [DIRECTORY]`

Flags:
- `-a, --all`: show hidden entries
- `-A, --almost-all`: show hidden entries except implied `.` and `..`
- `-l, --long`: long format
- `-H`: print column headers
- `--no-icons`: disable icons
- `--basic`: classic plain output
- `--sort <mode>`: `name | size | time`
- `-r, --reverse`: reverse sorting
- `--file-type`: append file type indicators, but do not append executable `*`
- `--author`: with `-l`, print author column
- `--theme <name>`
- `-h, --human-readable`: human-readable sizes
- `-?, --help`

Notes:
- Supports common built-in style flags (`-a`, `-l`, `-r`, `-t`, `-S`, `-h`, `--color`).
- Long format keeps aligned column margins.
- Long format timestamp: `DD Mon HH:MM`.
- `-a` includes implied `.` and `..` entries at top, while `-A` omits them.
- `--sort` supports: `name|size|time|owner|author|type|ext`.

## cat and bat Subcommands (Native Rust)

Usage:
- `dusk cat [OPTIONS] <FILE>...`
- `dusk cat [OPTIONS]` (stdin)
- `dusk bat [OPTIONS] [FILE]...` (pretty-first alias)

Flags:
- `-n, --number`
- `--no-number`
- `-p, --plain`
- `--pretty`
- `--theme <name>`
- `-b` (number nonblank)
- `-s` (squeeze blank lines)
- `-E` (show line end marker)
- `-T` (show tabs)
- `-h, --help`

Highlighting approach:
- Uses lightweight lexical/token highlighting by extension (no AST).
- Highlights comments, common keywords, numbers, and strings.
- Includes basic assembly token coloring (mnemonics, registers, immediates).

## dump Subcommand (Hex + Assembly)

Usage:
- `dusk dump [OPTIONS] <FILE>...`

Flags:
- `--hex`: show hex dump
- `--asm`: show assembly dump
- `--both`: show hex and assembly
- `--theme <name>`: set theme
- `-?, --help`

Notes:
- Hex mode is native Rust.
- Assembly mode requires `objdump` or `llvm-objdump` in `PATH` and returns clear errors if missing.

## git Subcommand

### Non-interactive

- `dusk git graph [theme]` and `dusk git log [theme]`:
  informative commit graph output.
- `dusk git status [theme]` and `dusk git viz [theme]`:
  staged/modified/untracked panel.

### Interactive TUI

- `dusk git tui [theme]`
- `dusk git interactive [theme]`

Core capabilities:
- stage/unstage selected and all files
- commit message entry and commit
- branch create and switch
- push current branch
- push to explicit remote branch
- upstream branch detection and display
- graph tab + commit-diff tab

Tabs:
- `1`: Workspace
- `2`: Graph
- `3`: CommitDiff

Primary keys:
- `j/k`, `Up/Down`: move/scroll
- `g/G`: top/bottom
- `h/l`, `Left/Right`, `Tab`: pane switch in workspace tab
- `s/u`: stage/unstage selected
- `A/U`: stage all/unstage all
- `c`: commit input mode
- `b/B`: create branch / switch branch input modes
- `p`: push current branch
- `R`: push remote branch input mode
- `t`: cycle theme
- `Ctrl+P` or `P`: command palette
- `:`: command mode
- `?`: centered help overlay
- `q`: quit

Input modes:
- commit: `commit msg: ...`
- branch create: `new branch: ...`
- branch switch: `switch branch: ...`
- remote push: `push remote branch: ...` (example `origin main`)
- command: `:...`

Command mode commands:
- `help`
- `cmdhelp` / `commands`
- `refresh` / `r`
- `stage`
- `unstage`
- `stage-all`
- `unstage-all`
- `commit <msg>`
- `push`
- `push-remote <remote>/<branch>`
- `push-remote <remote> <branch>`
- `branch <name>`
- `switch <name>`
- `workspace`
- `graph-tab` / `graphview`
- `commitdiff` / `commit-diff`
- `theme <name>`
- `themes`
- `palette`
- `quit` / `exit`

Palette:
- command search/filter
- cyclic selection
- Enter to execute
- includes theme switch entries

Mouse:
- scroll wheel support for active tab and palette

UI rendering:
- centered modal overlays (help/palette)
- compact layout for narrow terminals (`<100` columns)
- synchronized frame updates to reduce flicker
- blinking cursor in input/palette modes

Code organization:
- Commands are module-scoped by directory for easier refactoring:
  - `src/commands/cat/mod.rs`
  - `src/commands/diff/mod.rs`
  - `src/commands/git/mod.rs`
  - `src/commands/ls/mod.rs`
  - `src/commands/passthrough/mod.rs`
  - `src/commands/themes/mod.rs`
  - `src/commands/tree/mod.rs`
  - `src/commands/xtree/mod.rs`
- Git TUI internals:
  - `src/commands/git/tui/mod.rs`
  - `src/commands/git/tui/actions.rs`
  - `src/commands/git/tui/input.rs`
  - `src/commands/git/tui/render.rs`
- Convention for future commands:
  - Add new command implementations under `src/commands/<new-command>/`.
  - Keep shared utilities in `src/core/`.
  - Keep `src/app.rs` as command router only.

## diff Subcommand

- `dusk diff [theme] [--staged]`
- Side-by-side git diff with line numbers.

## themes Subcommand

- `dusk themes list`

## Wrapper Subcommands

- `dusk find [args...]` -> system `find`
- `dusk rg [args...]` -> `rg` if present, else system `grep`

## Theme Catalog

`default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark-pro | monokai | kanagawa | everforest | rose-pine | ayu | nightfox`

Default: `onedark-pro`

## OS Support

- Linux: supported
- macOS: supported
- Windows: supported

Cross-platform notes:
- ANSI output is TTY-aware and disabled for non-interactive output by default.
- Prompt export path uses system temp directory.
- Unix permission checks in `--audit` are limited on Windows.
- Symlink behavior depends on filesystem and permissions.

## Known Gaps / Non-goals

- `find`/`rg` remain wrappers rather than full native search-engine replacements.
- Clipboard auto-copy for prompt export is not implemented.

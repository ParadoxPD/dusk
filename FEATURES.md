# Dusk Features

`dusk` is a single CLI with native Rust features (`tree`, `ls`, `cat`, `xtree`, `git`, `diff`, `themes`) plus targeted wrappers (`find`, `rg`).

## Command Overview

- `dusk help`
- `dusk xtree [OPTIONS] [DIRECTORY]`
- `dusk git graph [theme]`
- `dusk git log [theme]`
- `dusk git status|viz [theme]`
- `dusk diff [theme] [--staged]`
- `dusk themes list`
- `dusk tree [args...]`
- `dusk find [args...]`
- `dusk rg [args...]`
- `dusk ls [args...]`
- `dusk cat [args...]`

## `xtree` (Native Rust)

Use this for the full enhanced tree/inspection/analysis workflow.

### Help

- `dusk xtree --help`
- `dusk xtree --tldr`

### Flags

Navigation:
- `-L <depth>`: limit recursion depth
- `-d`: directories only
- `-a`: include hidden files/directories
- `-e, --exclude <pattern>`: add exclusion pattern (repeatable)
- `-I <pattern>`: tree-compatible alias for exclude

Display:
- `-i`: show metadata (permission bits + modified timestamp)
- `-s`: hide file sizes
- `--no-icon`: disable Nerd Font icons
- `--theme <name>`: `default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark-pro | monokai | kanagawa | everforest | rose-pine | ayu | nightfox`
- `--tests`: highlight test files/directories
- `--count`: show per-directory file counts
- `--noreport`: hide the final summary line

Inspect files:
- `-c, --cat <ext...>`: print file content for matching extensions
- `-g, --grep <pattern>`: print up to first 5 matches per file
- `--clip <n>`: max printed lines per file (default: `100`)
- `--no-clip`, `--nc`: disable clipping

Filtering:
- `--no-git`: disable `.gitignore` filtering
- `--no-treeignore`: disable `.treeignore` filtering
- `--focus <ext...>`: only keep dirs/files containing matching extensions

Analysis:
- `--stats`: extension/language counts
- `--big`: mark files larger than 5 MB
- `--dupes`: detect duplicates via content hash
- `--audit`: basic security checks (permissions/executable/secrets)
- `--fingerprint`: project summary report

Organization:
- `--sort <mode>`: `name | size | time`
- `--group`: grouped output by extension
- `--resolve`: resolve symlink targets

Output formats:
- `--md`: markdown export
- `--json`: JSON export
- `--prompt`: AI-friendly dump, saved to OS temp dir as `tree_prompt.txt`

### Examples

- `dusk xtree`
- `dusk xtree -L 2 --theme nord`
- `dusk xtree -g TODO -c rs ts --clip 80`
- `dusk xtree --stats --big --dupes`
- `dusk xtree --fingerprint`
- `dusk xtree --json > tree.json`
- `dusk xtree --md > tree.md`
- `dusk xtree --prompt`

## `tree` Subcommand (Native Rust)

- `dusk tree [OPTIONS] [DIRECTORY]`
- Uses the same native engine as `xtree` (same flags/features).
- Quick help:
  - `dusk tree --tldr`
  - `dusk tree --help`

## `ls` Subcommand (Native Rust, eza-style)

### Usage

- `dusk ls [OPTIONS] [DIRECTORY]`

### Flags

- `-a, --all`: show hidden files
- `-l, --long`: long listing (perms, size, timestamp)
- `--no-icons`: disable Nerd Font icons
- `--basic`: classic plain output (no colors/icons)
- `--sort <mode>`: `name | size | time`
- `-r, --reverse`: reverse sorting
- `--theme <name>`: `default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark-pro | monokai | kanagawa | everforest | rose-pine | ayu | nightfox`
- `-h, --help`: help

### Examples

- `dusk ls`
- `dusk ls --basic`
- `dusk ls -la`
- `dusk ls -l --sort time`

## `cat` Subcommand (Native Rust, bat-style)

### Usage

- `dusk cat [OPTIONS] <FILE>...`
- `dusk cat [OPTIONS]` (reads stdin when no files are provided)
- `dusk bat [OPTIONS] [FILE]...` (pretty-first alias)

### Flags

- `-n, --number`: show line numbers
- `--no-number`: hide line numbers
- `-p, --plain`: disable color styling
- `--theme <name>`: `default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark-pro | monokai | kanagawa | everforest | rose-pine | ayu | nightfox`
- `-h, --help`: help
- `--pretty`: force bat-like pretty mode
- `-b`: number nonblank lines
- `-s`: squeeze repeated blank lines
- `-E`: show `$` at line end
- `-T`: show tabs as `^I`

### Examples

- `dusk cat src/main.rs`
- `dusk cat --no-number README.md`
- `dusk cat -p Cargo.toml`
- `printf 'a\\n\\n\\tb\\n' | dusk cat -n -E -T -s`
- `dusk bat src/main.rs`

## `git` Subcommand

### `dusk git graph [theme]`
Alias of `dusk git log [theme]`.

### `dusk git log [theme]`
Shows an informative graph history (`hash`, refs, subject, author, relative time) with VSCode-style focus.

### `dusk git status [theme]`
Shows branch + split panels for:
- staged
- modified
- untracked

### `dusk git viz [theme]`
Alias of `status`.

## `diff` Subcommand

### `dusk diff [theme] [--staged]`
Shows side-by-side git diff with old/new line numbers, optionally staged changes.

## `themes` Subcommand

### `dusk themes list`
Lists available theme names.

## Wrapper Subcommands

These pass arguments through to installed tools.

- `dusk find [args...]` -> system `find`
- `dusk rg [args...]` -> `rg` if installed, otherwise system `grep`

## OS Support Matrix

### Native features (`xtree`, `git`, `diff`, `themes`)

- Linux: supported
- macOS: supported
- Windows: supported

### Notes by feature

- ANSI color safety:
  - Color is enabled only for interactive terminals.
  - On non-TTY output, `TERM=dumb`, or when `NO_COLOR` is set, ANSI escapes are disabled.
  - This prevents escape-code leakage into pipes, redirected files, and copied plain text.
  - Override with `DUSK_COLOR=always` or `CLICOLOR_FORCE=1` when you explicitly want forced colors.

- Prompt export path:
  - Implemented cross-platform via system temp directory (`std::env::temp_dir()`).

- File permission security checks (`--audit`):
  - Linux/macOS: world-writable and executable checks are supported.
  - Windows: Unix permission bits do not exist in the same format. Dusk still runs secret-pattern scanning, but Unix-style permission checks are skipped.

- Symlink resolution (`--resolve`):
  - Works where the OS/filesystem exposes symlink metadata and resolution permissions.

- Native replacements:
  - `tree`/`ls`/`cat` are implemented from scratch in Rust, so they do not depend on `tree`, `eza`, or `bat`.
  - Icon rendering uses an expanded developer-focused Nerd Font icon library shared across commands.
  - `cat` behaves like basic cat by default; `bat` alias enables pretty mode by default.
  - `ls` accepts common built-in flags (`-a`, `-l`, `-r`, `-t`, `-S`, `-h`, `--color`).
  - Default theme is `onedark-pro` with high-visibility colors (gray/dim palette avoided).

- Wrappers that still depend on system binaries:
  - `find` and `rg` subcommands require `find` and (`rg` or `grep`) in `PATH`.

## Known Gaps / Explicit Non-Goals

- Dusk intentionally keeps `find`/`rg` as wrappers to avoid reimplementing full search engines.
- Clipboard copy for `--prompt` is not implemented yet; file output to temp dir is implemented for all OSs.

# FEATURES

This document outlines all `dusk` subcommands, flags, and compatibility notes.

## Subcommands

- `dusk help`
- `dusk xtree [OPTIONS] [DIRECTORY]`
- `dusk tree [OPTIONS] [DIRECTORY]` (alias of `xtree`)
- `dusk ls [OPTIONS] [FILE|DIR]...`
- `dusk eza [OPTIONS] [FILE|DIR]...` (alias of `ls`)
- `dusk cat [OPTIONS] [FILE]...`
- `dusk bat [OPTIONS] [FILE]...`
- `dusk git log|graph [theme]`
- `dusk git status|viz [theme]`
- `dusk git tui|interactive [theme]`
- `dusk diff [theme] [--staged]`
- `dusk dump [--hex|--asm|--both] [--theme <name>] <file>...`
- `dusk themes list`
- `dusk find [args...]`
- `dusk rg [args...]`
- `dusk grep [args...]`

## `xtree` / `tree`

- Help:
  - `dusk xtree --help`
  - `dusk xtree --tldr`
- Major options:
  - `-L <depth>`, `-d`, `-a`, `-e|--exclude <pattern>`, `-I <pattern>`
  - `-i`, `-s`, `--no-icon`, `--theme <name>`, `--tests`, `--count`, `--noreport`
  - `-c|--cat <ext...>`, `-g|--grep <pattern>`, `--clip <n>`, `--no-clip|--nc`
  - `--no-git`, `--no-treeignore`, `--focus <ext...>`
  - `--stats`, `--big`, `--dupes`, `--audit`, `--fingerprint`
  - `--sort <name|size|time>`, `--group`, `--resolve`
  - `--md`, `--json`, `--prompt`

## `ls` / `eza`

- Major options:
  - `-a|--all`, `-A|--almost-all`, `-l|--long`, `-H`, `-r|--reverse`
  - `-t`, `-S`, `-h|--human-readable`
  - `--file-type`, `--author`, `--sort <column>`
  - `--icons`, `--no-icons`, `--basic`
  - `--theme <name>`, `--color <auto|always|never>`
  - `-?`, `--help`
- Notes:
  - `-h` is human-readable size, not help.
  - `-a` includes implied `.` and `..`; `-A` excludes implied entries.
  - long format aligns columns and uses `DD Mon HH:MM` timestamp display.
  - `--sort` supports `name|size|time|owner|author|type|ext`.

## `cat` / `bat`

- Major options:
  - `-n`, `-b`, `-s`, `-E`, `-T`
  - `--pretty`, `--plain`/`-p`, `--no-number`, `--theme <name>`
  - `-h`, `--help`
- Notes:
  - `cat` defaults to plain mode.
  - `bat` defaults to pretty mode with lexical syntax highlighting.

## `git`

### Non-interactive

- `dusk git log` / `graph`: commit graph visualization.
- `dusk git status` / `viz`: grouped staged/modified/untracked status panel.

### Interactive TUI

- `dusk git tui` / `interactive`
- Tabs: Workspace, Graph, CommitDiff.
- Capabilities:
  - stage/unstage selected and all files
  - commit with message
  - create/switch branches
  - push current branch and push to explicit remote/branch
  - upstream branch display
  - theme cycle and theme selection via palette/commands
- Input/navigation:
  - Vim-style movement keys + arrow keys
  - command palette (`Ctrl+P`)
  - command mode (`:`) with `:cmdhelp`
  - centered help overlay (`?`)
  - mouse wheel scrolling

## `diff`

- `dusk diff [theme] [--staged]`
- Side-by-side git diff with line numbers and colorized hunks.

## `dump`

- `dusk dump [OPTIONS] <file>...`
- Options:
  - `--hex`
  - `--asm`
  - `--both`
  - `--theme <name>`
  - `--help`, `-?`
- Notes:
  - default mode is hex-only
  - asm output uses aligned columns (address/opcodes/mnemonic/operands)
  - asm mode requires `objdump` or `llvm-objdump`

## `themes`

- `dusk themes list`
- Shows available theme names (with swatches when color is enabled).

## Wrapper Commands

- `dusk find [args...]` -> system `find`
- `dusk rg [args...]` -> system `rg`; fallback to `grep`
- `dusk grep [args...]` -> same resolver as `rg`

## Theme Catalog

`default | nord | gruvbox | dracula | solarized | catppuccin | tokyonight | onedark-pro | monokai | kanagawa | everforest | rose-pine | ayu | nightfox`

Default theme: `onedark-pro`

## Platform Support

- Linux: supported
- macOS: supported
- Windows: supported

### Platform Notes

- ANSI output is TTY-aware and disables itself when redirected/piped unless forced.
- Some audit-style permission checks are less rich on Windows than Unix.
- Symlink behavior depends on OS/filesystem permissions.

## Dependency Guards

`dusk` fails fast with explicit errors when required external binaries are missing:

- `git` for `dusk git` and `dusk diff`
- `find` for `dusk find`
- `rg` or `grep` for `dusk rg`/`dusk grep`
- `objdump` or `llvm-objdump` for `dusk dump --asm`

## Detailed Command Docs

- [`docs/README.md`](docs/README.md)
- [`docs/help.md`](docs/help.md)
- [`docs/xtree.md`](docs/xtree.md)
- [`docs/ls.md`](docs/ls.md)
- [`docs/cat-bat.md`](docs/cat-bat.md)
- [`docs/git.md`](docs/git.md)
- [`docs/diff.md`](docs/diff.md)
- [`docs/dump.md`](docs/dump.md)
- [`docs/themes.md`](docs/themes.md)
- [`docs/wrappers.md`](docs/wrappers.md)

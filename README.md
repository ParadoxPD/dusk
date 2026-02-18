# dusk

![Rust](https://img.shields.io/badge/rust-2024%20edition-orange)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-blue)
![Theme](https://img.shields.io/badge/default%20theme-onedark--pro-2ea043)
![Commands](https://img.shields.io/badge/native%20commands-tree%20%7C%20ls%20%7C%20cat%20%7C%20xtree%20%7C%20git%20%7C%20diff-purple)

`dusk` is a single terminal toolkit that combines:

- native Rust reimplementations of `tree` / eza-style `ls` / bat-style `cat`
- enhanced project inspection (`xtree`) with themes, icons, stats, and exports
- git graph/status/diff views plus a full interactive TUI
- find/grep workflows (`find`, `rg` passthrough wrappers)
- terminal-capability-aware ANSI color behavior (no ANSI leakage in non-TTY output)

See `FEATURES.md` for complete flags and workflows.
See `TESTING.md` for tests, benchmarks, and coverage.

## Quick Start

```bash
cargo build --release
./target/release/dusk help

# interactive git panel
./target/release/dusk git tui
```

## Command Map

```bash
dusk help

dusk xtree [OPTIONS] [DIRECTORY]
dusk tree [OPTIONS] [DIRECTORY]
dusk ls [OPTIONS] [DIRECTORY]
dusk cat [OPTIONS] [FILE]...
dusk bat [OPTIONS] [FILE]...

dusk git log [theme]
dusk git graph [theme]
dusk git status [theme]
dusk git viz [theme]
dusk git tui [theme]

dusk diff [theme] [--staged]
dusk dump [--hex|--asm|--both] [--theme <name>] <file>...
dusk themes list

dusk find [args...]
dusk rg [args...]
```

## Git TUI Highlights

`dusk git tui` now includes:

- tabs: `Workspace`, `Graph`, `CommitDiff`
- interactive staging and unstaging (including untracked files)
- commit creation, branch create/switch, push workflows
- upstream visibility in header (shows tracking branch if present)
- remote push support to explicit target branch
- command palette (`Ctrl+P` / `P`) with filtering
- centered help overlay and command-mode help (`:cmdhelp`)
- vim-style navigation + mouse wheel scroll support
- blinking input cursor in command/input modes

## Themes

Available themes:

```text
default, nord, gruvbox, dracula, solarized, catppuccin, tokyonight, onedark-pro, monokai, kanagawa, everforest, rose-pine, ayu, nightfox
```

Default theme: `onedark-pro`

## Project Stats

- Language: Rust (Edition 2024)
- Native command families: 6 (`tree`, `ls`, `cat`, `xtree`, `git`, `diff`)
- Theme count: 14
- Interactive Git TUI modules: 4 (`mod.rs`, `actions.rs`, `input.rs`, `render.rs`)
- Test suites: unit tests + CLI integration tests

## Architecture Convention

- Every CLI command is implemented under `src/commands/<command>/` as a module directory.
- `src/app.rs` only routes commands and does not hold command implementation logic.
- Future commands should follow the same module-directory rule for easier refactoring and ownership boundaries.

## Color and Pipe Safety

- Colors are enabled only for interactive terminals by default.
- In non-TTY output, `TERM=dumb`, or when `NO_COLOR` is set, ANSI is disabled.
- This prevents escape-code leakage into redirected files and pipelines.
- Force color with `DUSK_COLOR=always` or `CLICOLOR_FORCE=1`.

## Compatibility Notes

- `tree`/`ls`/`cat` are native Rust implementations and do not shell out to `tree`, `eza`, or `bat`.
- `dump` is native for hex view; assembly mode uses `objdump`/`llvm-objdump` when available.
- `find` and `rg` are passthrough wrappers and require binaries in `PATH`.
- `rg` falls back to `grep` if `rg` is unavailable.
- `ls` updates:
  - `-h` is human-readable size (not help); use `--help` or `-?` for help.
  - `-H` enables column headers.
  - `-a` includes implied `.` and `..` entries at the top.
  - `-A` (`--almost-all`) includes hidden entries but omits implied `.` and `..`.
  - `--author` prints author column in long view.
  - `--file-type` appends file type markers without executable `*`.
  - `--sort` supports `name|size|time|owner|author|type|ext`.

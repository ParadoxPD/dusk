# dusk

![Rust](https://img.shields.io/badge/rust-2024%20edition-orange)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-blue)
![Theme](https://img.shields.io/badge/default%20theme-onedark--pro-2ea043)
![Coverage](https://img.shields.io/badge/tests-unit%20%2B%20cli-success)
![Docs](https://img.shields.io/badge/docs-command--level-informational)

`dusk` is a unified terminal toolkit that brings tree/find/grep/eza/bat-style workflows, git visualization, diff tooling, and binary dumping into one command.

## Highlights

- Native Rust implementations for `tree` (`xtree`), `ls` (`eza` alias), `cat`, and `bat`-style view
- Safe `rm` replacement with soft-delete trash, hard-delete flag, and interactive trash scanner TUI
- Safe wrappers for `mv`, `cp`, and `ln` with overwrite guard rails and sudo retry prompt (Unix)
- Native LOC reporting in `xtree` (`--loc`) and LOC-aware `--stats`
- Full-color theming with terminal-capability-aware ANSI behavior (no ANSI leakage on pipes/files)
- Git graph/status + interactive TUI with tabs, palette, overlays, staging/commit/push/branch operations
- Side-by-side git diff with line numbers
- Hex + assembly dump (`objdump`/`llvm-objdump` integration for ASM)
- Wrapper passthrough for `find` and `rg`/`grep` with explicit binary guard errors

## Quick Start

```bash
cargo build --release
./target/release/dusk help
```

## Commands

```bash
dusk help

dusk xtree [OPTIONS] [DIRECTORY]
dusk tree [OPTIONS] [DIRECTORY]
dusk ls [OPTIONS] [FILE|DIR]...
dusk eza [OPTIONS] [FILE|DIR]...
dusk rm [OPTIONS] FILE...
dusk mv [OPTIONS] SOURCE... DEST
dusk cp [OPTIONS] SOURCE... DEST
dusk ln [OPTIONS] TARGET LINK_NAME
dusk cat [OPTIONS] [FILE]...
dusk bat [OPTIONS] [FILE]...

dusk git log [theme]
dusk git status [theme]
dusk git diff [theme] [--staged] [--tui]
dusk git tui [theme]

dusk diff [theme] [--staged]
dusk dump [--hex|--asm|--both] [--theme <name>] <file>...
dusk themes list

dusk find [args...]
dusk rg [args...]
dusk grep [args...]
```

## Documentation

- Feature matrix: [`FEATURES.md`](FEATURES.md)
- Testing/bench: [`TESTING.md`](TESTING.md)
- Command docs index: [`docs/README.md`](docs/README.md)
- Per-command docs:
  - [`docs/help.md`](docs/help.md)
  - [`docs/xtree.md`](docs/xtree.md)
  - [`docs/ls.md`](docs/ls.md)
  - [`docs/cat-bat.md`](docs/cat-bat.md)
  - [`docs/rm.md`](docs/rm.md)
  - [`docs/mv-cp-ln.md`](docs/mv-cp-ln.md)
  - [`docs/git.md`](docs/git.md)
  - [`docs/diff.md`](docs/diff.md)
  - [`docs/dump.md`](docs/dump.md)
  - [`docs/themes.md`](docs/themes.md)
  - [`docs/wrappers.md`](docs/wrappers.md)

## Themes

Built-in themes:

`default`, `nord`, `gruvbox`, `dracula`, `solarized`, `catppuccin`, `tokyonight`, `onedark-pro`, `monokai`, `kanagawa`, `everforest`, `rose-pine`, `ayu`, `nightfox`

Default: `onedark-pro`

## Compatibility Notes

- Native Rust: `xtree/tree`, `ls/eza`, `cat/bat`, `diff`, `git` views/TUI, and hex dump rendering.
- External dependency for assembly mode: `objdump` or `llvm-objdump`.
- Wrapper commands: `find` and `rg`/`grep` use system binaries.
- Binary guards provide explicit error messages when required tools are unavailable.

# dusk

`dusk` is a single terminal tool that combines:

- native Rust reimplementations of `tree` / `eza` (`ls`) / `bat` (`cat`)
- `find` / `ripgrep` workflows
- a built-in enhanced tree engine (`xtree`) with themes + Nerd Font icons
- git graph visualization + git status panel
- colorful git diff output
- terminal-capability-aware ANSI coloring (no ANSI escape leakage in pipes/redirection)

See `FEATURES.md` for the complete command and flag reference.

## Commands

```bash
dusk xtree [xtree-options...]
dusk tree [tree-options...]
dusk find [find-options...]
dusk rg [ripgrep-options...]
dusk ls [native-ls-options...]
dusk cat [native-cat-options...]

dusk git log [theme]
dusk git graph [theme]
dusk git status [theme]
dusk diff [theme] [--staged]
dusk themes list
```

## Xtree Help

```bash
dusk xtree --tldr
dusk xtree --help
```

## Themes

```text
default, nord, gruvbox, dracula, solarized, catppuccin, tokyonight, onedark, monokai, kanagawa, everforest, rose-pine, ayu, nightfox
```

Default theme: `tokyonight`

## Notes

- `dusk tree`, `dusk ls`, and `dusk cat` are native Rust implementations (no `tree`, `eza`, or `bat` subprocesses).
- `dusk cat` defaults to basic `cat`-style plain output and supports stdin.
- `dusk bat` is an alias that defaults to pretty, themed, line-numbered output.
- `dusk ls` defaults to eza-style colorful/icon-rich output (TTY) and supports common `ls` flags (`-a`, `-l`, `-r`, `-t`, `-S`, `-h`, `--color`) with aligned long-format columns.
- `dusk ls --basic` switches to classic plain output.
- `dusk find` uses system `find`.
- `dusk rg` uses `rg` if installed, otherwise falls back to system `grep`.
- `dusk xtree` is implemented in pure Rust (no embedded shell script).
- `dusk diff` renders side-by-side output with old/new line numbers.
- `dusk git log` provides a rich commit graph view similar to VSCode-style history visuals.
- Colors are automatically disabled for non-interactive output (`NO_COLOR`, non-TTY, `TERM=dumb`), so redirected/piped output stays clean.

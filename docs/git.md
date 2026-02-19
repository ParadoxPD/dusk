# `dusk git`

Enhanced Git UX with both non-interactive views and an interactive TUI.

## Usage

```bash
dusk git log [theme]
dusk git status [theme]
dusk git diff [theme] [--staged] [--tui]
dusk git tui [theme]
dusk git interactive [theme]
```

Requires `git` in `PATH`.

## Non-interactive

- `log`: decorated history graph (VSCode-style overview)
- `status`: separated staged / modified / untracked sections
- `diff`: side-by-side line-numbered diff with syntax highlighting
  - terminal mode by default
  - isolated TUI mode with `--tui`

## Interactive TUI

Tabs:

- `1`: Workspace
- `2`: Graph
- `3`: CommitDiff

Features:

- stage/unstage selected file
- stage all / unstage all
- commit with message input
- create/switch branch
- push current branch
- push to explicit remote+branch target
- upstream branch visibility in status/header
- command palette (`Ctrl+P`)
- centered help overlay (`?`)
- command mode (`:`) and command help (`:cmdhelp`)
- mouse wheel scrolling
- compact layout fallback for narrow terminals

### Navigation Keys

- `j/k`, arrow up/down: move selection
- `h/l`, left/right, `Tab`: pane switch
- `g/G`: top/bottom
- `q`: quit

### Action Keys

- `s/u`: stage/unstage selected
- `A/U`: stage-all/unstage-all
- `c`: commit input mode
- `b/B`: create/switch branch input mode
- `p`: push current branch
- `R`: push to remote branch input mode
- `t`: cycle theme
- `Ctrl+P` or `P`: open command palette
- `:`: command mode
- `?`: help overlay

### Command Mode Commands

- `help`
- `cmdhelp`, `commands`
- `refresh`, `r`
- `stage`, `unstage`
- `stage-all`, `unstage-all`
- `commit <msg>`
- `push`
- `push-remote <remote>/<branch>`
- `push-remote <remote> <branch>`
- `branch <name>`
- `switch <name>`
- `workspace`
- `graph-tab`, `graphview`
- `commitdiff`, `commit-diff`
- `theme <name>`
- `themes`
- `palette`
- `quit`, `exit`

## Examples

```bash
dusk git log
dusk git status
dusk git tui
dusk git tui onedark-pro
```

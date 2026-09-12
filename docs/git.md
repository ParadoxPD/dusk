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
- `4`: Conflicts

Features:

- stage/unstage selected file
- stage all / unstage all
- commit with message input
- create branch + branch picker/switcher
- push current branch
- fetch all remotes (`--prune`)
- pull current branch (`--rebase`)
- push to explicit remote+branch target
- merge conflict resolver overlay (ours/theirs/mark-resolved/abort/mergetool)
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
- `b`: create branch input mode
- `B`: open branch picker/switcher
- `f`: fetch remotes (`git fetch --all --prune`)
- `L`: pull current branch (`git pull --rebase`)
- `m`: open merge conflict resolver for selected conflicted file
- `4`: open Conflicts tab
- Conflicts tab:
  - `Space`: mark/unmark file
  - `a`: mark-all / clear-all
  - `o/i`: resolve selected/marked with ours/theirs
  - `O/I`: resolve all conflicts with ours/theirs
  - `m`: mark selected/marked resolved (`git add`)
  - `x`: abort merge
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
- `fetch`
- `pull`
- `push-remote <remote>/<branch>`
- `push-remote <remote> <branch>`
- `branch <name>` (or `branch` to open picker)
- `branches`, `branch-picker`
- `switch <name>` (or `switch` to open picker)
- `resolve-conflict`, `resolve`
- `conflicts`, `conflict-tab`, `conflicts-tab`
- `resolve-all-ours`, `resolve-all-theirs`
- `resolve-marked-ours`, `resolve-marked-theirs`
- `mark-resolved`
- `abort-merge`
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

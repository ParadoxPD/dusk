# `dusk rm`

Safe `rm` replacement with soft-delete by default.

## Usage

```bash
dusk rm [OPTIONS] FILE...
dusk rm --trash-tui
```

## Behavior Model

- Default delete path: move target into trash (recoverable).
- Hard delete path: `--permanent` / `-P` removes directly.
- Directory delete requires `-r` / `-R` / `--recursive`.

## Drop-in Compatible Flags

- `-f`, `--force`: ignore missing paths and never prompt.
- `-i`, `--interactive`: ask before each remove.
- `-r`, `-R`, `--recursive`: remove directories recursively.
- `-v`, `--verbose`: print action per target.

## Safety/Trash Flags

- `-P`, `--permanent`, `--hard-delete`: bypass trash and delete directly.
- `--trash`: force soft-delete mode.
- `--trash-tui`, `--scan-trash`: open interactive trash scanner.
- `--restore <id|pattern>`: restore matching entries from trash.
- `--empty-trash`: permanently delete all trash entries.

## Trash Scanner TUI

- `j/k` or `Up/Down`: move selection
- `Space`: mark/unmark current entry
- `r`: restore selected/marked entries
- `d`: permanently delete selected/marked entries from trash
- `g/G`: jump top/bottom
- `q` / `Esc`: quit

## Examples

```bash
# Safe delete (to trash)
dusk rm file.txt

# Delete directory safely
dusk rm -r build/

# Permanent delete
dusk rm -rP build/

# Interactive confirmation
dusk rm -i secrets.env

# Open trash scanner
dusk rm --trash-tui

# Restore by id prefix or filename pattern
dusk rm --restore important

# Empty trash without prompt
dusk rm --empty-trash -f
```

## Platform Notes

- Linux: `$XDG_DATA_HOME/Trash/dusk` or `~/.local/share/Trash/dusk`
- macOS: `~/.Trash/dusk`
- Windows: `%LOCALAPPDATA%/dusk/Trash`
- Override for any OS: `DUSK_TRASH_DIR=/path/to/trash`

## Related

- [`docs/help.md`](help.md)
- [`docs/xtree.md`](xtree.md)

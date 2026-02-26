# `dusk help`

Top-level command router help for the full toolkit.

## Usage

```bash
dusk help
dusk --help
dusk -h
```

All three forms print the same top-level command map.

## What it Shows

- Native command groups and short descriptions
- Pass-through command group (`find`, `rg`)
- Quick-start examples
- Git/TUI entrypoints
- Dump and theme entrypoints

## Behavior Notes

- Help output is colorized when terminal color is enabled.
- Color is automatically suppressed when output is redirected/piped unless forced.
- `dusk help` is the canonical place to discover command families before opening command-specific help.

## Recommended Navigation Flow

```bash
dusk help
dusk xtree --tldr
dusk ls --help
dusk rm --help
dusk git --help
dusk dump --help
```

## Related Docs

- [`xtree/tree`](xtree.md)
- [`ls/eza`](ls.md)
- [`cat/bat`](cat-bat.md)
- [`rm`](rm.md)
- [`mv/cp/ln`](mv-cp-ln.md)
- [`git`](git.md)
- [`diff`](diff.md)
- [`dump`](dump.md)
- [`themes`](themes.md)
- [`wrappers`](wrappers.md)

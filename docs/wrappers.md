# Wrapper Commands (`find`, `rg`, `grep`)

These commands are thin wrappers that call system tools.

## Usage

```bash
dusk find [args...]
dusk rg [args...]
dusk grep [args...]
```

## Behavior

- `dusk find` requires system `find`.
- `dusk rg` uses `rg` if present.
- If `rg` is missing, it falls back to system `grep`.
- `dusk grep` also routes through the same `rg/grep` resolver.

## Binary Guards

If required binaries are missing, `dusk` prints explicit dependency errors instead of failing silently.

## Examples

```bash
dusk find . -name '*.rs'
dusk rg "TODO" src
```

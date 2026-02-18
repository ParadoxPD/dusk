# `dusk ls` (`dusk eza` alias)

Native Rust listing with eza-style colors/icons and ls-compatible flags.

## Usage

```bash
dusk ls [OPTIONS] [FILE|DIR]...
dusk eza [OPTIONS] [FILE|DIR]...
```

## Core Flags

- `-a, --all`: include hidden files and implied `.` / `..`
- `-A, --almost-all`: include hidden files except implied `.` / `..`
- `-l, --long`: long listing format
- `-H`: print long-format headers
- `-h, --human-readable`: human-readable sizes in long mode
- `-r, --reverse`: reverse sort
- `-t`: sort by mtime
- `-S`: sort by size
- `--sort <column>`: `name|size|time|owner|author|type|ext`
- `--author`: with `-l`, show author column
- `--file-type`: append file type marker, but no executable `*`

## Color, Icons, Themes

- `--icons`: enable icons (default)
- `--no-icons`: disable icons
- `--basic`: plain, classic output (no icons/colors)
- `--theme <name>`: set theme
- `--color <when>`: `auto|always|never`

## Help

- `--help` or `-?`

Note: `-h` is human-readable size, not help.

## Examples

```bash
dusk ls
dusk ls -laH --author
dusk ls -l --sort ext
dusk ls -l --file-type --human-readable
dusk ls --basic -A
```

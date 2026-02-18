# `dusk xtree` and `dusk tree`

`tree` is an alias of `xtree`.

## Usage

```bash
dusk xtree [OPTIONS] [DIRECTORY]
dusk tree [OPTIONS] [DIRECTORY]
```

## TL;DR

```bash
dusk xtree --tldr
dusk xtree -L 2 --theme onedark-pro
dusk xtree --stats --loc --big --dupes
dusk xtree --json > tree.json
```

## Main Flags

### Navigation

- `-L <depth>`: limit recursion depth
- `-d`: directories only
- `-a`: include hidden files
- `-e, --exclude <pattern>`: exclude pattern (repeatable)
- `-I <pattern>`: tree-compatible exclude alias

### Display

- `-i`: show metadata
- `-s`: hide file size info
- `--no-icon`: disable Nerd Font icons
- `--theme <name>`: set theme
- `--tests`: highlight test files
- `--count`: show file count per directory
- `--noreport`: hide final totals line

### Inspection

- `-c, --cat <ext...>`: print content for matching extensions
- `-g, --grep <pattern>`: print first matches inside files
- `--clip <n>`: cap printed lines per file
- `--no-clip` / `--nc`: disable clipping

### Filtering

- `--no-git`: ignore `.gitignore` rules
- `--no-treeignore`: ignore `.treeignore`
- `--focus <ext...>`: keep directories/files with matching extensions

### Analysis

- `--stats`: extension/language statistics
- `--loc`: total lines of code (LOC) summary
- `--big`: mark large files (>5MB)
- `--dupes`: duplicate detection by content hash
- `--audit`: security checks (permissions + secret hints)
- `--fingerprint`: project summary snapshot

### Organization

- `--sort <mode>`: `name|size|time`
- `--group`: group by extension
- `--resolve`: show resolved symlink target

### Output modes

- `--md`: markdown export
- `--json`: JSON export
- `--prompt`: AI-friendly project dump

## Examples

```bash
dusk xtree
dusk xtree -L 3 --count --tests
dusk xtree -g TODO -c rs toml
dusk xtree --loc
dusk xtree --focus rs ts --sort time
dusk xtree --fingerprint --dupes
```

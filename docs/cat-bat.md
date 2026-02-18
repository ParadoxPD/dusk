# `dusk cat` and `dusk bat`

Single implementation with two modes:

- `dusk cat`: plain mode by default
- `dusk bat`: pretty mode by default

## Usage

```bash
dusk cat [OPTIONS] [FILE]...
dusk bat [OPTIONS] [FILE]...
```

Reads stdin when no files are provided.

## Cat-Compatible Flags

- `-n`: number all lines
- `-b`: number nonblank lines only
- `-s`: squeeze repeated blank lines
- `-E`: show `$` at end of lines
- `-T`: show tabs as `^I`

## Pretty/Theme Flags

- `--pretty`: force pretty mode
- `--plain`, `-p`: force plain mode
- `--no-number`: disable line numbers in pretty mode
- `--theme <name>`: select theme
- `--help`, `-h`: show help

## Highlighting Model

`dusk bat` uses lightweight lexical highlighting (token-based, not AST-based):

- comments
- strings
- numbers
- common keywords
- assembly tokens (mnemonics/register/immediates)

## Examples

```bash
dusk cat Cargo.toml
dusk cat -n src/main.rs
dusk bat src/main.rs
dusk bat --theme monokai --no-number src/lib.rs
printf 'a\n\n\n b\n' | dusk cat -s
```

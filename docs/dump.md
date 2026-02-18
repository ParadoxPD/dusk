# `dusk dump`

Hex + assembly dumper with aligned assembly columns.

## Usage

```bash
dusk dump [OPTIONS] <file>...
```

## Flags

- `--hex`: hex only
- `--asm`: assembly only
- `--both`: both hex and assembly
- `--theme <name>`: set theme
- `--help`, `-?`: show help

Default mode is `--hex` when no explicit mode is given.

## Assembly Rendering

Assembly output is aligned to columns:

- address
- opcode bytes
- mnemonic
- operands

This matches objdump-style readability more closely.

## Requirements

- Hex mode: native Rust
- ASM mode: needs `objdump` or `llvm-objdump` in `PATH`

## Examples

```bash
dusk dump --hex target/release/dusk
dusk dump --asm target/release/dusk
dusk dump --both --theme nightfox target/release/dusk
```

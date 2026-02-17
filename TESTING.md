# Testing and Benchmarking

## Test Suite

Run all tests:

```bash
./scripts/test.sh
```

or directly:

```bash
cargo test
```

What is covered:
- Unit tests for style/theme/config parser behavior
- CLI integration tests for command pathways:
  - `help`
  - `themes list`
  - `ls --help` and `ls --basic`
  - `cat` stdin/plain behavior
  - `bat` pretty behavior
  - `xtree --help` and tree compatibility flags
  - `tree --noreport`
  - `diff --help`
  - `git --help`
  - unknown command failure

## Benchmarks

Run command-path benchmarks:

```bash
./scripts/bench.sh
```

or directly:

```bash
cargo build --release
cargo bench
```

Bench coverage includes representative pathways for:
- `help`
- `ls` default and long output
- `cat` plain and `bat` pretty
- `tree` and `xtree --json`
- `git --help`
- `diff --help`
- `themes list`

## Coverage (100% gate)

To enforce a strict 100% line coverage gate:

```bash
./scripts/coverage.sh
```

This uses `cargo-llvm-cov`:

```bash
cargo install cargo-llvm-cov
```

Notes:
- The gate is strict (`--fail-under-lines 100`) and will fail until all lines are covered.
- For this codebase (terminal rendering, subprocess paths, platform branches), reaching true 100% generally requires additional targeted tests and/or refactoring for testability.

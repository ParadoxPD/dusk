# `dusk mv` / `dusk cp` / `dusk ln`

Safe wrappers around system `mv`, `cp`, and `ln`.

## Goals

- Stay compatible with common Unix usage.
- Add overwrite guard rails by default.
- Offer optional sudo retry prompt when ownership mismatch is detected (Unix).

## Usage

```bash
dusk mv [OPTIONS] SOURCE... DEST
dusk cp [OPTIONS] SOURCE... DEST
dusk ln [OPTIONS] TARGET LINK_NAME
```

## Shared Safety Behavior

- Existing target conflict:
  - prompts before overwrite unless `-f/--force`.
  - respects `-n/--no-clobber` by skipping conflicting operations.
- Ownership guard (Unix):
  - if path ownership differs from current user, asks whether to retry with `sudo`.
- External binary guard:
  - explicit error if `mv`, `cp`, or `ln` is missing in `PATH`.

## `mv`

- Uses system `mv` for actual operation.
- Common flags supported and passed through (`-f`, `-i`, `-n`, `-v`, `-t`, long forms).

## `cp`

- Uses system `cp` for actual operation.
- Common flags supported and passed through (`-f`, `-i`, `-n`, `-v`, `-r/-R`, `-t`, long forms).

## `ln`

- Uses system `ln` for actual operation.
- Common flags supported and passed through (`-s`, `-f`, `-i`, `-n`, `-v`, `-t`, long forms).
- If source/target are missing, prompts interactively for source and target paths.

## Examples

```bash
# move with guard rails
dusk mv a.txt b.txt

# copy recursively with confirmation on overwrite
dusk cp -r src/ backup/

# symlink creation
dusk ln -s target.txt link.txt

# no-clobber copy
dusk cp -n a.txt b.txt
```

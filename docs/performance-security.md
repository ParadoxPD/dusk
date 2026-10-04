# Performance and Security

## File reads

`ls`, `cat`, and `xtree` currently use synchronous filesystem operations. Directory entries and metadata are read as they are needed, while file content is read synchronously for commands that explicitly inspect it.

This is deliberate for terminal commands:

- output remains deterministic and correctly ordered;
- file descriptor usage stays bounded;
- a slow or unreadable path does not create an unbounded queue;
- small directory listings do not pay thread scheduling overhead.

The safe optimization for very large repositories is bounded concurrency, not "as many file descriptors as possible." A future implementation can use a fixed worker count and a bounded queue for independent metadata, hashing, or syntax classification work. It must preserve display order, cap file reads, skip permission failures, and stop scheduling work when the output consumer stops reading. Unbounded parallel reads can exhaust descriptors, evict useful filesystem cache, and make spinning disks slower.

## Process delegation

External tools are invoked with `std::process::Command` and argument arrays. Dusk does not build shell command strings or invoke a shell, so paths and user input are passed as literal arguments rather than interpreted as shell syntax.

The shared PATH guard checks that a discovered Unix command is a regular file with at least one execute bit. Windows uses the platform executable extensions. Features that require an optional binary report a targeted error when it is unavailable.

The Git TUI also checks Git exit status and surfaces stderr. Destructive operations such as hard reset, squash, trash deletion, and conflict actions are explicit UI actions; reset and squash require confirmation. `.gitignore` writes are repository-relative and refuse traversal, absolute paths, control-line input, and symlinked `.gitignore` files.

Some delegated behavior is intentionally controlled by the user's system configuration. For example, `git mergetool` may launch the configured merge tool. Dusk does not reinterpret its arguments, but that tool's own configuration remains a system-level trust boundary.

## Current limitations

- File contents are not read concurrently yet. This avoids changing output semantics and keeps memory use predictable.
- `dump --asm` uses `objdump` or `llvm-objdump` when available because implementing a complete multi-format disassembler is outside the scope of a small terminal utility.
- Git operations still require the `git` executable and inherit Git's repository, credential, hook, and remote configuration.

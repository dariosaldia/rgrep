# rgrep

A small Rust implementation of a `grep`-like CLI tool, built as a learning exercise. The focus is correctness, error-handling, and testability (TDD + integration tests), not full feature parity or maximum performance.

## What it does

- Recursively scans files under a root path.
- Prints matches in a `grep`-like format:

  `path:line_number:line_contents`

- Continues scanning even if some files fail to open/read.
- Skips likely-binary files (sniffing first N bytes).
- Skips hidden files/dirs during traversal (Unix-style `.` prefix), except when the user explicitly passes a hidden root path.
- Scans files in parallel and guards printing to avoid interleaved output.

## Usage

Build and run:

```bash
cargo run -- <pattern> <path>
```

Example:

```bash
cargo run -- Hello .
```

## Exit codes

- `0` — at least one match found and no errors occurred
- `1` — no matches found and no errors occurred
- `2` — at least one error occurred (even if matches were found)

## Tests

Run all tests:

```bash
cargo test
```

Notes:

- Unit tests cover helpers (sniffing/hidden detection/walker behavior).
- Integration tests under `tests/` treat the project as a black-box CLI and assert both output and exit codes.

## Project layout

- `src/main.rs` — thin CLI entry point
- `src/lib.rs` — orchestration layer (`run(...) -> i32`)
- `src/scanner.rs` — file scanning + line matching + printing
- `src/sniff.rs` — binary/text sniffing logic
- `src/walker.rs` — directory traversal + hidden handling + file collection
- `tests/` — CLI integration tests

## Current behavior details

### Hidden paths

- “Hidden” on Unix: any file/dir name starting with `.`.
- When scanning from a visible root directory, hidden entries inside it are skipped.
- If the user explicitly passes a hidden file/dir as the root argument (e.g., `./.env`, `./.git`), it is scanned.

### Binary detection

- Reads up to 4096 bytes from the start of the file.
- Treats a file as “binary” if a NUL byte is present or if the sampled bytes are not valid UTF-8.
- If the file is considered text, the scanner rewinds and performs line-based scanning.

## Next steps

Pick one of these directions.

### A) Feature work

- Add `--regex` end-to-end (CLI parsing, matching, tests).
- Add flags:
  - `-i` / `--ignore-case`
  - `--hidden` (include hidden entries during traversal)
  - `--count` (print counts instead of lines)

### B) Engineering depth

- Improve error reporting structure (typed errors / categories).
- Reduce syscalls/allocations during traversal and scanning.
- Add benchmarks on large directory trees.
- Improve output formatting behavior (e.g., deterministic ordering vs parallel speed).

### C) Polish + freeze

- Clean commit history (remove WIP commits, squash, tag a stable version).
- Add a changelog / release notes.

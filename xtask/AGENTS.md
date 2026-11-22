# AGENTS.md (`xtask`)

Additive to root [`AGENTS.md`](../AGENTS.md).

## 0. Invariants (CRITICAL)

1. **Idempotent writes.** Always write generated files through `util::write_if_changed` to preserve timestamps and Cargo incremental caches when content is unchanged.
2. **Location-independent execution.** Resolve the workspace root via `util::find_workspace_root()`; never assume the caller's working directory.

## 1. Identity

- **Crate:** `xtask` (`xtask/`), internal CLI binary (`publish = false`), default workspace member.
- **Role:** workspace automation, orchestration, and all filesystem, git, and network I/O.

## 2. Design & Constraints

- **Subcommand isolation:** Subcommands with specific invariants define an additive `AGENTS.md` in their module directory.

## 3. Architecture

- `src/main.rs` parses CLI arguments (`clap`) and dispatches to subcommand modules under `src/commands/`.

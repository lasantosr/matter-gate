# AGENTS.md

Workspace rules for AI agents. Each crate and subsystem has an additive `AGENTS.md`; read it before editing that scope. Nearer files refine, never contradict, this file.

## 0. Invariants (CRITICAL)

1. **Never pass `--workspace` to `cargo`.** Memory-heavy crates are excluded from `default-members`; `--workspace` builds them all at once and can OOM-kill the host. Use default members, `-p <crate>`, or the crate's `just` recipes.
2. **Use `just` recipes over raw commands.** Run `just --list`. Pass recipe arguments (e.g., `version`, `features`) rather than crafting hand-made `cargo` commands; fall back to raw `cargo` only when no recipe fits.
3. **Format with `just fmt`.** Runs `cargo +nightly fmt --all` (never plain `cargo fmt`) and `taplo fmt`.
4. **Fix lints at the root.** Clippy must pass with `-Dwarnings` on the `just` validation recipes (`just clippy`, `just spec-clippy`), where full recipe feature sets are enabled (one version at a time). Do **not** contort code or add `#[allow]`/`#[expect]` to pass `-Dwarnings` across arbitrary unsupported partial feature subsets that naturally produce unused warnings. Clippy subsumes `cargo check`; do not run both.
5. **Never hand-edit generated code.** Modify the generator, then regenerate.
6. **Tests travel with behavior.** Add or update colocated `#[test]`s for every behavior or logic change.
7. **Declare dependencies once** in root `[workspace.dependencies]`:
   - Object form with explicit `version`; never string shorthand.
   - Pin to the latest major (or `0.x`) line with the minimum minor/patch version that satisfies the APIs and features used, so downstream crates can resolve freely.
   - Core only (no feature flags at the root). Set `default-features = false` when a crate enables multiple default features.
   - Members inherit with `dep = { workspace = true }` and opt into features locally. Dotted `dep.workspace = true` is forbidden in every dependencies table.
8. **Order items top-down for human readability.** Place module-level `type` aliases and `const` declarations first (even when private), then exported items (`pub` → `pub(crate)` → `pub(super)`) ordered by importance and conciseness (e.g., options/input structs, entry-point functions, error enums), followed by private helpers in call/reading order, with `#[cfg(test)] mod tests` at the bottom. Colocate `impl` blocks with their type (exported methods first): for a single type use one `impl` block; for multiple types, colocate each type's exported `impl` with its definition and place private helper `impl` blocks below in reading order.
9. **Use Conventional Commits**, scoped by crate for single-crate changes: `feat(<crate>): ...`.
10. **Keep `AGENTS.md` durable and scoped.** Record only invariants, contracts, and workflows for that file's scope. Never restate parent rules, assert rules for other crates, or list files/types.

## 1. Identity

- **Project:** `matter-gate`, a single-binary Matter bridge engine and specification suite in Rust for single-board computers and home-lab servers.
- **Current scope:** Matter specification and domain model crates (`spec/`) and build/codegen orchestration (`xtask/`).
- **Role:** principal Rust systems engineer prioritizing zero-cost abstractions, deterministic behavior, memory efficiency, and compile-time safety.
- **Error policy:** `thiserror` in libraries; `anyhow` in CLI/orchestrators.

## 2. Architecture

| Directory | Role |
| :--- | :--- |
| `spec/` | Matter wire data model, versioned specification models, and in-memory code generator. |
| `xtask/` | Workspace orchestration, codegen runner, and all I/O. |

- **Public ecosystem boundaries:** Crates whose `AGENTS.md` designates them as public boundaries must maintain SemVer stability (`#[non_exhaustive]` on evolving public structs/enums, paired with constructors or builders).
- **Workspace-internal crates:** All other crates prefer exhaustive matching and destructuring (no `#[non_exhaustive]`) so changes surface as compile errors at every call site.
- **API conventions (`Optional` vs `Option`, `set_*` vs `add_*`):** Use `Optional<T>` for Matter protocol presence/omission and `Option<T>` for Rust API mechanics (lookup misses, previous values on `set_*`/`remove_*`, type casts); use `set_*` for keyed upserts/scalar replacements returning `Option<Previous>` and `add_*` for ID-set insertions returning `bool`.

## 3. Workflow

| Action | Command |
| :--- | :--- |
| Pre-commit gate (deps, TOML, fmt, clippy) | `just pre-commit` |
| Format Rust and TOML | `just fmt` |
| Lint default members | `just clippy` |
| Test default members | `just test` |
| Build docs (warnings denied) | `just doc` |
| Check dependency declarations | `just check-deps` |

Scope verification to what changed. Do not run unnecessary checks:

- **Docs, Markdown (`.md`), and Config (`.yaml`, `.yml`):** Never run Rust formatting (`cargo fmt`), codegen, compilation, or tests. If TOML files changed, run `taplo fmt`.
- **Single crate changes:** Prefer the crate's `just` recipes (e.g., `just spec-clippy` / `just spec-test` for `matter-spec`); otherwise run only the owning crate's scoped check (`cargo test -p <crate>`, `cargo clippy -p <crate> -- -Dwarnings`).
- **Specification generator changes:** Run `just spec-codegen` and formatting only when editing generator code (`xtask/src/commands/codegen_spec/` or `spec/matter-spec-gen/`) or spec schemas.
- **Full pre-commit gate (`just pre-commit`):** Run only when modifying cross-cutting Rust code, workspace manifests, or dependencies.

Execution steps:

1. Locate the owning crate and read its `AGENTS.md`.
2. Classify the boundary (public SemVer boundary vs workspace-internal).
3. Implement minimal changes with colocated `#[test]`s when changing behavior.
4. Run the scoped verification matching the modified files.

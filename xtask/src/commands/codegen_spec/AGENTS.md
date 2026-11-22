# AGENTS.md (`xtask codegen-spec`)

Additive to [`xtask/AGENTS.md`](../../../AGENTS.md).

## 0. Invariants (CRITICAL)

1. **Offline by default.** Read XML from `xtask/spec-data/`; access the network only when `--fetch` or `--pin` is passed.
2. **Orchestrate, do not parse or model.** Delegate XML parsing, type resolution, and item rendering to [`matter-spec-gen`](../../../../spec/matter-spec-gen/AGENTS.md).
3. **Prune stale outputs.** Remove obsolete generated `.rs` files and version directories so removed upstream definitions never linger.
4. **Format after generation.** Always run `cargo +nightly fmt -p matter-spec` and `taplo fmt spec/matter-spec/Cargo.toml` at the end of codegen.
5. **Idempotent generation.** Generating twice in a row must modify no files.

## 1. Identity

- **Module:** `xtask/src/commands/codegen_spec/` (`just spec-codegen`, `just spec-list`).
- **Role:** fetch/cache CSA XML schemas from `project-chip/connectedhomeip` and assemble [`matter-spec`](../../../../spec/matter-spec/AGENTS.md).

## 2. Design & Constraints

- **Responsibility boundary:** Manages git sparse-checkout caching, module/manifest layout, file emission, stale-output pruning, and formatting—never XML parsing or Rust AST modeling.
- **Generated crate layout:** Emits `__shared/` buckets (for items shared across 2+ versions), versioned `v<major>_<minor>` modules (no `latest` alias), `dispatch` submodules with `> 64B` threshold-boxed variants, top-level `src/lib.rs`, and the `[features]` table in `spec/matter-spec/Cargo.toml`.

## 3. Architecture

- Fetches and caches upstream CSA XML in `xtask/spec-data/` (with per-`major.minor` cache pruning).
- Loads cached XML into `SpecInput`, invokes `matter_spec_gen::build` and category reconciliation, emits the crate layout, prunes stale outputs, and formats the result.

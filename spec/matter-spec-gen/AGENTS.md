# AGENTS.md (`matter-spec-gen`)

Additive to root [`AGENTS.md`](../../AGENTS.md).

## 0. Invariants (CRITICAL)

1. **Pure in-memory library.** Zero filesystem, network, git, CLI, or `anyhow` dependencies.
2. **Fail fast on unknown schema elements.** Unmapped scalar types, invalid boolean semantics, unknown command sources, unmapped event priorities, and unknown categories must return typed `thiserror` variants; never fall back silently.
3. **One-directional layering.** `xml` → `model` → `render` / `items` → `build`. Lower modules must never import higher modules; parse XML once.
4. **Layout agnostic.** Never write files, choose directory structures, emit `mod.rs` trees, or edit `Cargo.toml` manifests.
5. **Deterministic output.** Identical inputs must produce byte-identical `genco` token streams and rendered strings.

## 1. Identity

- **Crate:** `matter-spec-gen` (`spec/matter-spec-gen`), internal library (`publish = false`), default workspace member.
- **Role:** compile CSA ZAP XML definitions into Rust models conforming to [`matter-spec`](../matter-spec/AGENTS.md).

## 2. Design & Constraints

- **Flat layout & direction-split commands:** Emit flat cluster structs (with spec-optional fields as `Optional<T>` and `.with_*` builders), sparse patch structs, single-attribute enums, and direction-split `Command` (`ClientToServer`) vs. `CommandResponse` (`ServerToClient`) using non-generic owned decode/encode kernels.
- **Content-addressed `__shared` deduplication:** Hash canonical ASTs, referenced globals, and conformance/access/layout metadata transitively so items shared across 2+ versions are emitted once into `__shared`.
- **Static `SpecRegistry` & dispatch metadata:** Emit sorted static `SpecRegistry` tables and `> 64B` `TypeLayout` boxing metadata for `dispatch`.
- **Fixture tests:** Add or update XML fixture tests in `tests/` for every parser, model, or renderer change.

## 3. Architecture

- Accepts in-memory XML via `SpecInput` and `XmlDocument`, runs `build` (paired with cross-version category reconciliation), and returns a typed `Spec`.
- Rendered code items implement the public `Render` trait.

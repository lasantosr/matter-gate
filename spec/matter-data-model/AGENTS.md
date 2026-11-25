# AGENTS.md (`matter-data-model`)

Additive to root [`AGENTS.md`](../../AGENTS.md).

## 0. Invariants (CRITICAL)

1. **Pure wire data model.** Include only version-independent Matter Core Data Model entities, wire primitives/identifiers, and Interaction Model status codes.
2. **Keep `#![no_std]` + `alloc`.** Never import `std` unconditionally or add dependencies that require it.
3. **Zero runtime coupling.** No async runtimes, network stacks, web frameworks, or protocol engines.
4. **Maintain the public SemVer boundary.** Mark evolving public structs and `MatterStatus` `#[non_exhaustive]`, annotate pure queries/constructors/builders/wrappers `#[must_use]`, and keep `#![warn(missing_docs)]` clean.

## 1. Identity

- **Crate:** `matter-data-model` (`spec/matter-data-model`), publishable library, default workspace member.
- **Role:** stable wire types, identifiers, and status codes shared across the ecosystem.

## 2. Design & Constraints

- **Scope exclusions:** Exclude IM protocol framing (message/path Information Blocks, session/exchange IDs), versioned cluster/device schemas, derived unit wrappers, and static conformance/access/range metadata.
- **Typed structural fields vs. flat attribute bags:** Promote properties to typed struct fields only for closed, container-level topology or per-entity wire metadata that has no parallel entry inside an inner `Vec<Attribute>` (e.g., endpoint topology on `Endpoint`, `data_version` on `Cluster`, `event_number`/`timestamp` on `Event`). Keep anything addressable by `AttributeId` (including §7.13 global attributes `0xFFF8..=0xFFFD`) solely in `Cluster::attributes` with typed helper accessors—duplicating attribute-bag entries as struct fields creates split-brain state and bloats instances when omitted.
- **Zero-alloc read views:** Keep read accessors and views zero-allocation (`&T`, `&[T]`, `Optional<&str>`, or single-pass lazy iterators such as `ResultListIter<'_, T>`); callers opt into heap allocation only when calling `.to_vec()` or `.collect()`.
- **Infallible builders & type-only `TryFrom`:** Domain structs use `pub` fields and infallible constructors/`with_*` builders; `#[repr(transparent)]` ID newtypes keep their inner value private behind `.raw()`. Owned `TryFrom<AttributeValue>` moves payloads without cloning and checks wire type, enum discriminant, and field presence only (never spec value/length constraints): return `ConstraintError` (`0x87`) on missing mandatory fields or duplicate `FieldId`s, while tolerating distinct unknown `FieldId`s on TLV structs.
- **Exhaustive TLV primitives:** Keep `AttributeValue` intentionally exhaustive over Matter's fixed TLV primitive set.
- **Serde & JSON Schema:** Model wire omission (`Optional`, non-nullable) separately from wire `null` (`Nullable`). Always pair `#[serde(default)]` with `#[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]`.
- **Feature hygiene:** Keep `bytes` unconditional; keep `std`, `serde`, and `schemars` optional.

## 3. Architecture

- Domain concepts live in private modules (colocating each wire struct with its identifier newtypes) and are re-exported flatly at the crate root; serde helpers live under `serde`.
- Order `impl` methods on domain structs consistently: constructors, `with_*` builders in field order, then accessors grouped by field in field order (`has_*`, getter, `*_mut`, `set_*`/`add_*`, `remove_*`).

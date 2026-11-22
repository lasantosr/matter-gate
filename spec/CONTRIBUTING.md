# Contributing to the Matter Specification Crates

This guide covers the crates in `spec/` and their codegen tooling in `xtask/`. For repository setup, commit conventions, and the general workflow, start with the [root contributing guide](../CONTRIBUTING.md).

## Layout

| Path | Contents |
| :--- | :--- |
| `spec/matter-data-model` | Stable wire types, identifiers, and status codes |
| `spec/matter-spec` | Generated, versioned cluster and endpoint models |
| `spec/matter-spec-gen` | In-memory CSA XML code generator |
| `xtask` | Code generation orchestrator (`cargo xtask codegen-spec`) and cached CSA XML (`xtask/spec-data/`) |

[`spec/README.md`](README.md) explains why `matter-data-model` and `matter-spec` are separate crates.

## Guidelines

- **`matter-data-model` is a stability boundary.** Mark evolving public types `#[non_exhaustive]`, pair non-exhaustive structs with constructors or builders, and avoid breaking changes. Keep it `#![no_std]` + `alloc`, with no async runtime, networking, or protocol-stack dependencies.
- **The other crates are not stability boundaries.** Prefer exhaustive matching so that changes surface at every call site. Generated specification enums are exhaustive by design.
- **`matter-spec-gen` performs no I/O.** Filesystem access, networking, and git operations belong in `xtask`.

## Working on `matter-spec`

> [!WARNING]
> `matter-spec` contains multiple full specification versions, and building them together can exhaust memory. Work on **one version at a time, in a single cargo process, with `--jobs 1`**. The `just` recipes do this automatically.

```bash
just spec-clippy          # lint the latest version (or pass a version, e.g. v1_4)
just spec-test            # test the latest version (or pass a version, e.g. v1_4)
just spec-clippy-all      # lint every version, sequentially
just spec-test-all        # test every version, sequentially
```

Each run takes a minute or two.

## Code Generation

Most of `matter-spec` is generated; never edit generated files by hand. Change the generator (`spec/matter-spec-gen` or `xtask/src/commands/codegen_spec`), regenerate, and verify one version:

```bash
just spec-codegen                  # regenerate offline from cached XML
just spec-list                     # list cached specification versions
just spec-codegen --fetch          # fetch the latest upstream specs (requires network)
just spec-codegen --pin <version>  # pin a specific four-part upstream release (requires network)
just spec-codegen --fetch --force  # force re-download even if already cached (works with --fetch or --pin)
```

Generation is deterministic and idempotent: running it twice produces no changes. Commit regenerated files together with the generator change that caused them.


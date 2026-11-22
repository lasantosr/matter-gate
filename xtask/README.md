# xtask

Internal CLI binary powering workspace automation, build orchestration, and code generation for `matter-gate`.

Following the standard [`cargo-xtask` pattern](https://github.com/matklad/cargo-xtask), commands can be invoked directly through Cargo or via shorthand [`Justfile`](../Justfile) recipes.

## Usage

```bash
# Direct cargo invocation
cargo xtask <COMMAND> [OPTIONS]

# Or via just recipes
just spec-codegen [OPTIONS]
just spec-list [OPTIONS]
```

## Commands

### `codegen-spec`

Orchestrates the autogeneration of strongly-typed Matter domain models (`matter-spec`) from official Connectivity Standards Alliance (CSA) XML cluster specifications.

* **Offline by default**: Reads local cached XML schemas stored under [`spec-data/`](spec-data/).
* **Orchestration only**: Delegates XML parsing and Rust syntax rendering to [`matter-spec-gen`](../spec/matter-spec-gen), focusing solely on network caching, disk layout, file emission, and formatting.
* **Network-capable**: Can optionally fetch latest upstream tags or pin specific specification versions from the upstream [`connectedhomeip`](https://github.com/project-chip/connectedhomeip) repository.

#### Common Examples

```bash
# Regenerate matter-spec from existing cached XML definitions (fully offline)
cargo xtask codegen-spec
# List locally cached specification versions without regenerating code
cargo xtask codegen-spec --list
# Query upstream repository and list available remote specification tags
cargo xtask codegen-spec --list --fetch
# Fetch the latest Matter specifications from upstream and regenerate code
cargo xtask codegen-spec --fetch
# Fetch and pin a specific specification version (e.g., v1.4.0.0)
cargo xtask codegen-spec --pin "1.4.0.0"
```

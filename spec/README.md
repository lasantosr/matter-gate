# Matter Specification & Domain Model Suite

**The Matter protocol, modeled in type-safe Rust.**

This suite turns the official Connectivity Standards Alliance (CSA) [Matter](https://csa-iot.org/all-solutions/matter/) specification into ergonomic, compile-time-checked Rust types, from raw wire identifiers all the way up to stateful device endpoints.

Every model crate is `#![no_std]` compatible (`alloc`) and completely independent of any async runtime, network stack, or web framework. Whether you are building bare-metal accessory firmware, a multi-protocol bridge, a controller, or a conformance test harness, you get the full Matter data model with zero runtime lock-in.

## Crates

| Crate | Description |
| :--- | :--- |
| [`matter-data-model`](matter-data-model) | Stable wire hierarchy, typed identifiers, and Interaction Model status codes. |
| [`matter-spec`](matter-spec) | Generated, versioned clusters and device endpoints, feature-gated by Matter version, product domain, endpoint, or single cluster. |
| [`matter-spec-gen`](matter-spec-gen) | Workspace-internal pure in-memory code generator that compiles CSA XML schemas into Rust models. |

Each crate's README covers its features and usage examples.

## Why This Suite Exists (vs. `rs-matter-dm-clusters`)

Official stack-integrated codegen such as [`rs-matter`](https://github.com/project-chip/rs-matter)'s `rs-matter-dm-clusters` is purpose-built for firmware and device handlers tightly coupled to `rs-matter`'s TLV codecs and Interaction Model traits. That design is ideal when a single binary owns both the network stack and every cluster implementation.

Bridges, multi-protocol gateways, driver SDKs, controllers, and conformance tools face a different architectural problem:

1. **Decoupling domain data from binary TLV & IM runtime machinery:** Device drivers, plugins, and application libraries should be able to construct, transform, and exchange Matter state without pulling in a full network stack, TLV encoder/decoder, or async runtime, or breaking whenever the underlying stack changes.
2. **Preventing diamond-dependency conflicts:** Protocol stacks and drivers need a long-term stable shared vocabulary (`matter-data-model`) that does not break every time the CSA publishes a new Matter release. Keeping versioned cluster schemas in `matter-spec` lets `matter-spec` track new CSA releases freely while libraries and driver SDKs expose only `matter-data-model` in their public APIs.
3. **Bridging plain data to any Interaction Model stack:** A decoupled runtime or stack adapter (targeting `rs-matter` or any alternative stack) can receive plain data-model payloads from drivers and query `matter-spec`'s static registry by cluster and member ID for everything the Interaction Model layer needs: access control privileges, access and quality flags, cluster revisions, command response mappings, and zero-allocation conformance validation.

## Architecture: Why Two Model Crates?

Protocol libraries often tangle two things that evolve at completely different speeds:

1. **Wire grammar**. These define how Matter represents state and rarely change.
2. **Specification catalogs** of clusters, attributes, commands, events, and device types. The CSA revises and expands these with every Matter release.

Bundling both into a single crate forces breaking releases of the foundation whenever a cluster catalog updates, splitting the ecosystem across incompatible versions. Separating them solves this cleanly: `matter-data-model` stays a rock-solid shared vocabulary, while `matter-spec` tracks specification releases and converts to and from the wire types via standard `From` and `TryFrom` implementations.

| | `matter-data-model` | `matter-spec` |
| :--- | :--- | :--- |
| **Stability** | Long-term stable foundation; breaking changes avoided | Evolves with each Matter specification release |
| **Source** | Hand-written | Generated from the official CSA specification |
| **Intended scope** | Public Rust library APIs, pass-through crates, driver SDKs | Internal business logic for accessories, bridges, and drivers |

### Boundary Guidelines

- **Public Rust APIs:** Pass-through crates, transports, and driver SDKs should expose only `matter-data-model` in their public Rust signatures. This prevents diamond-dependency conflicts and keeps downstream crates interoperable regardless of which Matter specification versions they enable.
- **Internal application logic:** Accessories, drivers, and bridges use `matter-spec` internally for strong typing, sparse delta updates, thin wire metadata, and zero-alloc conformance validation, converting to and from `matter-data-model` at their public boundaries with zero-clone owned conversions.
- **Web and JSON APIs:** Exposing serialized `matter-spec` types over HTTP, WebSockets, or IPC is safe. Polymorphic, threshold-boxed `dispatch` enums and self-describing wrappers let you serialize and route heterogeneous clusters and commands cleanly. By default, Serde ignores unknown fields so external consumers and dashboards remain backwards-compatible across specification updates without coupling to a specific crate version. Services can either keep this default or enable the opt-in `deny-unknown-fields` cargo feature on leaf binaries when strict schema enforcement is required.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the [Apache License, Version 2.0](../LICENSE).

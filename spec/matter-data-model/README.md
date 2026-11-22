# matter-data-model

[![crates.io](https://img.shields.io/crates/v/matter-data-model.svg)](https://crates.io/crates/matter-data-model)
[![docs.rs](https://docs.rs/matter-data-model/badge.svg)](https://docs.rs/matter-data-model)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/lasantosr/matter-gate/blob/main/LICENSE)

**The universal, zero-bloat wire vocabulary for Matter in Rust.**

`matter-data-model` gives you the core [Matter](https://csa-iot.org/all-solutions/matter/) hierarchy (nodes, endpoints, clusters, attributes, commands, events, typed identifiers, and Interaction Model status codes) as clean, embedded-ready data structures.

## Why `matter-data-model`?

Protocol stacks, bridges, and drivers need a shared language to exchange Matter state without pulling in an entire network engine or breaking whenever a new cluster specification lands.

`matter-data-model` is that foundation. It models the Connectivity Standards Alliance (CSA) Data Model and Interaction Model status codes as plain data: no custom traits, no schema conformance validation, no message or path Information Blocks, and zero dependencies on async runtimes or transport stacks.

Need strongly typed, versioned cluster and endpoint models? Pair it with [`matter-spec`](https://crates.io/crates/matter-spec) for seamless conversions to and from these wire types.

## Features

- **Complete wire hierarchy:** Clean data structures, fluent builders, and in-place mutation helpers for Matter nodes, endpoints (including parts and semantic tags), clusters, attributes, commands, and events.
- **Zero-cost typed identifiers:** Transparent newtypes for node, endpoint, cluster, and attribute IDs (with Manufacturer Extension Identifier helpers) that catch raw integer mix-ups at compile time.
- **Universal wire values:** Covers every Matter primitive and collection, with first-class support for optional and nullable values and ergonomic combinators that work seamlessly with `matter-spec` types.
- **Normative status codes:** Direct, byte-accurate mappings for Interaction Model status codes.
- **Embedded & `no_std` ready:** Compiles identically on bare-metal microcontrollers (using `alloc`) and full Linux hosts.

## Cargo Features

| Feature | Default | Description |
| :--- | :---: | :--- |
| `std` | Yes | Standard library support. |
| `serde` | No | `Serialize` and `Deserialize` implementations for all types (targets self-describing keyed formats). |
| `schemars` | No | `JsonSchema` generation via `schemars` (enables `std`). |
| `deny-unknown-fields` | No | Disallow unrecognized fields in Serde deserialization and Schemars JSON schemas (takes effect when `serde` or `schemars` is enabled). **Library crates must NOT enable this feature**, leave it for leaf binary crates to opt in. |

## Examples

```rust
use matter_data_model::*;

// Application-specific identifiers (from Matter Core Spec / Application Cluster Spec / Device Library)
const ROOT_NODE_DEVICE_TYPE: DeviceTypeId = DeviceTypeId::new(0x0016);
const ON_OFF_LIGHT_DEVICE_TYPE: DeviceTypeId = DeviceTypeId::new(0x0100);
const BASIC_INFORMATION_CLUSTER: ClusterId = ClusterId::new(0x0028);
const ON_OFF_CLUSTER: ClusterId = ClusterId::new(0x0006);

// Assemble a Matter node with a Root Node endpoint (0) and an On/Off Light endpoint (1).
let node = Node::new(NodeId::new(0x0000_0000_1234_5678), Vec::new())
    .with_endpoint(Endpoint::single_device(
        EndpointId::ROOT,
        ROOT_NODE_DEVICE_TYPE,
        1,
        vec![Cluster::empty(BASIC_INFORMATION_CLUSTER)],
        Vec::new(),
    ))
    .with_endpoint(Endpoint::single_device(
        EndpointId::new(1),
        ON_OFF_LIGHT_DEVICE_TYPE,
        2,
        vec![
            Cluster::empty(ON_OFF_CLUSTER)
                .with_data_version(DataVersion::new(1))
                .with_attribute(AttributeId::new(0), true),
        ],
        Vec::new(),
    ));

assert_eq!(node.endpoints.len(), 2);
assert!(node.root_endpoint().is_some());

let light = node.endpoint(EndpointId::new(1)).unwrap();
let on_off = light.server_cluster(ON_OFF_CLUSTER).unwrap();
assert_eq!(
    on_off.attribute(AttributeId::new(0)),
    Some(&AttributeValue::Bool(true))
);
```

## License

Licensed under the [Apache License, Version 2.0](https://github.com/lasantosr/matter-gate/blob/main/LICENSE).

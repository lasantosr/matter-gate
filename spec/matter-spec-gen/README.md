# matter-spec-gen

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/lasantosr/matter-gate/blob/main/LICENSE)

**From official CSA Matter XML to strongly typed Rust, purely in memory.**

`matter-spec-gen` is the workspace-internal (`publish = false`) AST compiler and code generator behind [`matter-spec`](../matter-spec/). It transforms raw CSA cluster and device-type XML schemas into compile-time-checked Rust models.

## Overview

`matter-spec-gen` parses Connectivity Standards Alliance (CSA) cluster and device-type XML definitions and compiles them into strongly typed Rust token streams and source strings targeting [`matter-data-model`](../matter-data-model/). Because it performs zero I/O and makes no assumptions about directory layout, orchestrators such as `xtask` stay in full control of where schemas come from and how generated code is written.

## Features

- **Pure in-memory.** Zero filesystem, network, git, or CLI dependencies. Pass XML documents as in-memory strings (`SpecInput`, `XmlDocument`) and receive structured model items back.
- **Fail-fast validation.** Unmapped scalar types, invalid boolean semantics, unknown command sources, or unrecognized event priorities surface immediately as typed errors instead of silent fallbacks.
- **Deterministic rendering.** Every generated item implements `Render` (emitting `genco` Rust token trees or formatted source text), guaranteeing byte-identical output for identical inputs.
- **Layout agnostic.** Decides nothing about files, directories, `mod.rs` trees, or `Cargo.toml` feature tables. Your tooling owns the output structure.

## Cargo Features

This crate has no optional feature flags.

## Examples

```rust
use matter_spec_gen::{Render, SpecInput, XmlDocument, build};

let docs = vec![
    XmlDocument::new("chip-types.xml", chip_types_xml),
    XmlDocument::new("global-attributes.xml", global_attrs_xml),
    XmlDocument::new("on-off.xml", on_off_cluster_xml),
];

// Pass a version label (for naming) and the upstream release tag the XML comes from.
let input = SpecInput::new(version_label, release_tag, docs);
let spec = build([input]).expect("failed to build spec models");

for version in &spec.versions {
    for cluster in &version.clusters {
        println!("Cluster: {} ({:?})", cluster.name(), cluster.category());
        let rust_code = cluster.render().expect("failed to render cluster");
    }
}
```

## License

Licensed under the [Apache License, Version 2.0](https://github.com/lasantosr/matter-gate/blob/main/LICENSE).



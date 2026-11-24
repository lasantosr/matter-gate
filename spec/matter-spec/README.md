# matter-spec

[![crates.io](https://img.shields.io/crates/v/matter-spec.svg)](https://crates.io/crates/matter-spec)
[![docs.rs](https://docs.rs/matter-spec/badge.svg)](https://docs.rs/matter-spec)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/lasantosr/matter-gate/blob/main/LICENSE)

**Every Matter cluster, attribute, command, event, and device type: strongly typed at compile time.**

`matter-spec` brings the official Connectivity Standards Alliance (CSA) [Matter](https://csa-iot.org/all-solutions/matter/) specification to Rust with exhaustive enums, distinct attribute newtypes, stateful endpoints, conformance validation, and ergonomic unit helpers.

## Why `matter-spec`?

Working with raw cluster IDs and untyped wire values is error-prone. `matter-spec` gives you compile-time-checked models generated directly from the official CSA specification, organized into one module per supported Matter release.

It is completely independent of any network stack, transport, or async runtime. Its only Matter dependency is [`matter-data-model`](https://crates.io/crates/matter-data-model), which it re-exports and converts to and from seamlessly.

## Features

- **Lean typed models & sparse patches:** Every cluster provides a flat state struct with fluent builders, a sparse patch type for delta updates, a single-attribute enum, and direction-split command and response enums.
- **Lossless wire wrappers:** Always-enabled wrappers preserve data versions, global attributes, event numbers, timestamps, and Manufacturer Extension Identifier (MEI) extra attributes without bloating your domain structs.
- **Zero-clone owned conversions:** Owned wire payloads (strings, byte buffers, and vectors) convert into domain models with zero intermediate clones.
- **Exhaustive enums & bitmaps:** Cluster enums are exhaustive for their Matter release so matches are compiler-verified. Bitmaps provide named flag constants, bitwise operators, and set helpers. Error enums are also exhaustive by design (see [Exhaustive Error Enums & SemVer](#exhaustive-error-enums--semver)).
- **Zero-alloc conformance validation:** Static read-only specification tables power non-mutating, zero-allocation full-snapshot and partial-delta validation, alongside zero-alloc typed endpoint inspection.
- **Semantic unit & time newtypes:** Physical unit, time, and address types (temperature, percentage, power, epochs, dates, IP addresses, and more) with optional/nullable combinators and optional `chrono`, `time`, and `uom` bridges.
- **Boundary dispatch & dynamic schemas:** Feature-gated boundary enums for dynamic dispatch, self-describing JSON wrappers, and feature-map-pruned JSON schemas.
- **Embedded & `no_std` ready:** Compiles on bare-metal targets (using `alloc`) when default features are disabled, including the static registry and serialization.

## Cargo Features

Everything is feature-gated so you only compile what your project actually uses:

### 1. Matter Version (Required)

Enable at least one version feature (`v1_X`, e.g. `v1_4`); building without one is a compile error. Multiple versions can be enabled side by side with explicit imports. Global attributes, shared types, and extensions are available as soon as a version is selected. When a cluster or global type is identical across multiple Matter releases, those versions re-export the exact same Rust type, so values can be used interchangeably across versions without conversion.

### 2. Clusters and Endpoints

Features are layered from coarse to fine. Pick the narrowest tier that fits your use case to keep compile times fast and binaries small:

| Tier | Feature / Variable Pattern | Enables |
| :--- | :--- | :--- |
| All | `all` | Every product domain and unattached cluster |
| Product domain (recommended) | `appliances`, `cameras`, `closures`, `energy-management`, `general`, `hvac`, `lighting`, `measurement-and-sensing`, `media`, `network-infrastructure`, `robots` | The domain's endpoints and required clusters |
| Unattached clusters | `unattached-clusters` | Standalone and cross-cutting clusters not bound to a specific device type |
| Single endpoint | `endpoint-<name>` (e.g. `endpoint-on-off-light`) | One endpoint and its required clusters |
| Single cluster (env) | `MATTER_SPEC_CLUSTER_<NAME>=1` (e.g. `MATTER_SPEC_CLUSTER_ON_OFF=1`) | One cluster via build environment or `.cargo/config.toml` `[env]` |

### 3. Capabilities

| Feature | Default | Description |
| :--- | :---: | :--- |
| `std` | Yes | Standard library support across `matter-spec`, `matter-data-model`, `bytes`, and `serde`. |
| `registry` | No | `#![no_std]` + `alloc` static `SpecRegistry` and zero-alloc `Validator` conformance validation. |
| `dispatch` | No | Runtime `*Meta` traits, threshold-boxed boundary enums, and `Described<T>` / `Invocation<T>` self-describing JSON wrappers. |
| `serde` | No | `#![no_std]` + `alloc` `Serialize` and `Deserialize` implementations for all models. |
| `schemars` | No | Dynamic `FeatureMap`-pruned `JsonSchema` generation via `schemars` (enables `std` and `registry`). |
| `deny-unknown-fields` | No | Disallow unrecognized fields in Serde deserialization and Schemars JSON schemas (takes effect when `serde` or `schemars` is enabled). **Library crates must NOT enable this feature**, leave it for leaf binary crates to opt in. |
| `chrono` | No | `chrono` conversions for Matter time types. |
| `time` | No | `time` crate conversions for Matter time types. |
| `uom` | No | `uom` SI unit conversions for physical quantity types in `types`. |
| `testing` | No | Matter test and example clusters. |

## Architecture & Conversion Matrix

`matter-data-model` provides the untyped **wire / data-model types**, while `matter-spec` provides versioned, strongly-typed domain models unified by always-enabled `*Spec` traits.

| Entity | `matter-data-model` ↔ `matter-spec` | Lossless? | Notes |
| :--- | :--- | :---: | :--- |
| **Attribute** | `model::Attribute` ↔ `AttributeSpec` | Yes | Bare domain type carries full attribute ID & value. |
| **Command** | `model::Command` ↔ `CommandSpec` | Yes | Bare domain type carries full command ID & fields. |
| **Cluster** | `model::Cluster` ↔ `<Name>Cluster` | Domain-only | Bare struct omits `data_version`, `GlobalAttributes`, and MEI `extra_attributes`. |
| **Cluster (Wire)** | `model::Cluster` ↔ `WireCluster<T, F>` | **Yes (100%)** | Preserves `data_version`, `GlobalAttributes`, and MEI `extra_attributes`; `Deref`s to `T`. |
| **Event** | `model::Event` ↔ `EventSpec` | Domain-only | Bare struct omits `event_number`, `timestamp`, and `priority`. |
| **Event (Wire)** | `model::Event` ↔ `WireEvent<T>` | **Yes (100%)** | Preserves `event_number`, `timestamp`, and `priority`; `Deref`s to `T`. |
| **Endpoint** | `model::Endpoint` ↔ `DeviceTypeSpec` | Projection | `DeviceType` is a typed builder; use `ep.view_as::<D>()` (`EndpointExt`) for zero-alloc lossless inspection. |

## Examples

- **Single-version apps:** `use matter_spec::v1_4::prelude::*;` gives direct access to `OnOffCluster`, `Dishwasher`, `Validator`, `DataVersion`, `Temperature`, `WireCluster`, `EndpointExt`, and more.
- **Multi-version apps:** `use matter_spec::prelude::*;` gives explicit `v1_4::OnOffCluster`, `v1_6::OnOffCluster`, etc.

### Temperature Conversion, `Optional<Nullable<T>>`, and Bitmaps

```rust
use matter_spec::v1_4::prelude::*;
use thermostat::{RelayStateBitmap, attributes::OccupiedCoolingSetpoint};

// Construct an attribute from a semantic Temperature
let setpoint = OccupiedCoolingSetpoint::new(Temperature::from_celsius(21.5));
assert_eq!(setpoint.raw(), 2150); // Matter stores hundredths of °C via Deref

// Access Temperature helpers directly via Deref
let fahrenheit = setpoint.to_fahrenheit();
assert!((fahrenheit - 70.7).abs() < 0.1);

// Transform Optional<Nullable<T>> fields cleanly
let opt_temp: Optional<Nullable<Temperature>> = Optional::Present(Nullable::NonNull(*setpoint));
assert_eq!(opt_temp.map_nonnull(Temperature::to_celsius).as_nonnull(), Some(&21.5));

// Encode to the wire representation
let wire_val: AttributeValue = setpoint.into_value();
assert_eq!(wire_val, AttributeValue::I16(2150));

// Combine bit flags with standard operators
let relay = RelayStateBitmap::HEAT | RelayStateBitmap::FAN;
assert!(relay.contains(RelayStateBitmap::HEAT));
```

### Sparse Delta Patches & Lossless Wire Wrappers

```rust
use matter_spec::v1_4::prelude::*;

let mut state = OnOffCluster::new(on_off::OnOff::Off);
let mut patch = OnOffClusterPatch::default();
patch.apply_attribute(OnOffCluster::ATTR_ON_OFF, true.into()).unwrap();
state.apply_patch(patch);
assert_eq!(state.on_off, on_off::OnOff::On);

// Preserve instance-level wire metadata alongside the lean domain state
let wire = state.into_wire().with_data_version(DataVersion::new(42));
assert_eq!(wire.on_off, on_off::OnOff::On); // direct field access via Deref
let model_cluster = Cluster::from(wire);
assert_eq!(model_cluster.data_version, Optional::Present(DataVersion::new(42)));
```

### Endpoints, Mode Clusters, Dispatch & Zero-Alloc Validation

```rust
use matter_spec::v1_4::prelude::*;

// Build a mode cluster in one line from semantic tags
let mode = dishwasher_mode::DishwasherModeCluster::from_tags(
    [dishwasher_mode::ModeTag::Normal, dishwasher_mode::ModeTag::Heavy],
    dishwasher_mode::ModeTag::Normal,
)
.unwrap();

let dishwasher = Dishwasher::new(operational_state::OperationalStateCluster::new(
    None,
    None,
    vec![operational_state::OperationalStateEnum::Running.into()],
    operational_state::OperationalStateEnum::Running,
    operational_state::ErrorStateEnum::NoError,
))
.with_on_off(on_off::OnOff::On)
.with_dishwasher_mode(mode);

assert!(dishwasher.on_off.is_present());

// Convert to the wire model, inspect via EndpointExt, and round-trip back
let ep = dishwasher.clone().into_endpoint(EndpointId::new(1));
let view = ep.view_as::<Dishwasher>().unwrap().unwrap();
assert!(view.operational_state().is_some());
let roundtrip = Dishwasher::try_from(ep).unwrap();
assert_eq!(dishwasher, roundtrip);

// Self-describing JSON wrapper for commands (requires `dispatch`)
let _invocation = on_off::commands::Toggle.invoke_on(1);

// Zero-alloc validation against the static specification registry (requires `registry`)
let light_ep = OnOffLight::new(
    identify::IdentifyCluster::default(),
    groups::GroupsCluster::default(),
    scenes_management::ScenesManagementCluster::default(),
    on_off::OnOff::Off,
)
.into_endpoint(EndpointId::new(2));
Validator::full().endpoint(&light_ep).unwrap();
```

## Exhaustive Error Enums & SemVer

All error enums in `matter_spec::error` (`AttributeError`, `CommandError`, `EventError`, `ClusterError`, `EndpointError`, `NodeError`) intentionally omit `#[non_exhaustive]`. Adding a new failure variant is treated as an intentional breaking change so downstream crates that match exhaustively are notified at compile time. Callers that want forward-compatible handling can match with a trailing `_ =>` arm or map errors to `model::MatterStatus` via `MatterStatus::from(err)`.

## License

Licensed under the [Apache License, Version 2.0](https://github.com/lasantosr/matter-gate/blob/main/LICENSE).

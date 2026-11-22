# MatterGate

**One binary. Any hardware. Native Matter.**

MatterGate is a [Matter](https://csa-iot.org/all-solutions/matter/) bridge engine written in Rust: a single, dependency-free binary engineered for speed and a tiny memory footprint, so it runs effortlessly on single-board computers and home-lab servers alike. Its driver system lets you bridge third-party devices and protocols into Apple Home, Google Home, and Home Assistant without the runtime bloat or diamond-dependency headaches of traditional plugin ecosystems.

> [!NOTE]
> MatterGate is under active development. The bridge engine and driver SDK are not published yet.

## Matter Specification Crates

This repository also hosts standalone Rust crates that model the Matter protocol from the ground up. Generated from the official Connectivity Standards Alliance (CSA) definitions and decoupled from any async runtime or network stack, they drop cleanly into accessories, bridges, controllers, and custom tooling.

Explore [`spec/`](spec/README.md) to see more.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

## License

Licensed under the [Apache License, Version 2.0](LICENSE). Specification XML inputs are derived from the Connectivity Standards Alliance (CSA) [`connectedhomeip`](https://github.com/project-chip/connectedhomeip) repository (Apache-2.0).

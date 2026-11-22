# Contributing to MatterGate

Thanks for your interest in contributing! Bug reports, fixes, tests, and documentation improvements are all welcome. For larger changes, please open an issue first so we can agree on the approach before you invest time.

Working on the Matter specification crates? Also read [`spec/CONTRIBUTING.md`](spec/CONTRIBUTING.md).

## Getting Set Up

You need a recent stable Rust toolchain, a nightly toolchain with `rustfmt` (used only for formatting), [`just`](https://github.com/casey/just), and [`taplo`](https://github.com/tamasfe/taplo).

```bash
git clone https://github.com/lasantosr/matter-gate.git
cd matter-gate
git config core.hooksPath .githooks   # run pre-commit checks automatically
just test                             # confirm everything works
```

Run `just` with no arguments (or see [`Justfile`](Justfile)) to list every available task.

## Development Workflow

1. Create a branch from `main`.
2. Make your change, with tests.
3. Run `just pre-commit`, and `just test` & `just doc` where relevant.
4. Open a pull request.

> [!WARNING]
> Never pass `--workspace` to `cargo`. Some crates are excluded from default members because building all specification versions at once can exhaust memory. Prefer the [`Justfile`](Justfile) recipes over hand-crafted `cargo` commands, or scope with `-p <crate>`.

## Conventions

- **Formatting:** run `just fmt` (runs nightly `rustfmt` and `taplo fmt`; never plain `cargo fmt`).
- **Lints:** clippy must pass with `-Dwarnings`. Fix the underlying issue instead of silencing the lint with `#[allow]` or `#[expect]`.
- **Tests:** add or update colocated `#[test]`s with every behavior change.
- **Dependencies:** declare each one once in the root `Cargo.toml` under `[workspace.dependencies]`, in object form with an explicit `version` (never string shorthand) and no `features` flags (core-only; set `default-features = false` when a crate enables multiple default features). Member crates inherit with `dep = { workspace = true }` (never dotted `dep.workspace = true` and never standalone `version = "..."`) and opt into features locally. `just check-deps` enforces all four rules.
- **Generated code:** never edit it by hand. Change the generator and regenerate (see [`spec/CONTRIBUTING.md`](spec/CONTRIBUTING.md)).

## Commits and Pull Requests

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `<type>(<scope>): <summary>`.

- **Types:** `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- **Scope:** the crate name when a change touches a single crate, for example `feat(matter-data-model): add GroupId`. Omit it for cross-cutting changes.
- Mark breaking changes with `!` or a `BREAKING CHANGE:` footer.

For pull requests:

- Keep each one focused, and explain what changed and why.
- All CI checks must pass.
- Update the relevant README when behavior or usage changes.
- Leave `CHANGELOG.md` files alone; they are generated at release time.

## Releases

Releases are automated with [`release-plz`](https://release-plz.dev). From the commit history it opens a release PR that bumps versions and updates each crate's changelog. Merging it publishes to crates.io and creates `<crate>-v<version>` tags. CI also runs `cargo-semver-checks` on pull requests.

## License

By contributing, you agree that your contributions are licensed under the [Apache License, Version 2.0](LICENSE).

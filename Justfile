# Justfile for matter-gate workspace tasks
# Usage: `just <recipe>` or `just` to list available recipes

# Execute recipes with PowerShell by default on Windows
set windows-powershell

# Latest Matter specification version targeted by single-spec recipes
latest := "v1_6"
# Full capability feature set for single-spec recipes
feat := "all,registry,dispatch,serde,schemars,jiff,time,uom"
test_feat := "testing," + feat

# Default recipe: display available recipes in source order
default:
    @just --list --unsorted

# -----------------------------------------------------------------------------
# Validation
# -----------------------------------------------------------------------------

# Run workspace pre-commit checks (TOML, dependency format, Rust format, clippy)
[group('validation')]
pre-commit: check-deps check-toml check-fmt clippy
    @echo "==> Pre-commit checks passed successfully!"

# Validate that Cargo.toml dependency declarations adhere to workspace invariants
[group('validation')]
check-deps: _check-deps

# Check TOML formatting via Taplo
[group('validation')]
check-toml: (_require "taplo --version" "cargo install taplo-cli")
    taplo fmt --check

# Check Rust code formatting using Nightly toolchain
[group('validation')]
check-fmt: (_require "cargo +nightly fmt --version" "rustup toolchain install nightly --component rustfmt")
    cargo +nightly fmt --all -- --check

# Lint default members using Clippy (strict: zero warnings)
[group('validation')]
clippy:
    cargo clippy --locked --all-targets --all-features -- -Dwarnings

# Check documentation build with no warnings and all features
[group('validation')]
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features

# Run unit and integration tests across default members
[group('validation')]
test:
    cargo test --locked --all-features

# -----------------------------------------------------------------------------
# Formatting
# -----------------------------------------------------------------------------

# Format all workspace files (Rust and TOML)
[group('format')]
fmt: fmt-rust fmt-toml

# Format all Rust files using Nightly toolchain
[group('format')]
fmt-rust: (_require "cargo +nightly fmt --version" "rustup toolchain install nightly --component rustfmt")
    cargo +nightly fmt --all

# Format all TOML files via Taplo
[group('format')]
fmt-toml: (_require "taplo --version" "cargo install taplo-cli")
    taplo fmt

# -----------------------------------------------------------------------------
# Matter Specification Domain Models
# -----------------------------------------------------------------------------

# Lint a single matter-spec version
[arg('check', long, value='true')]
[group('matter-spec')]
spec-clippy version=latest features=feat check="false":
    cargo {{ if check == "true" { "check" } else { "clippy" } }} -p matter-spec --locked {{ if check == "true" { "" } else { "--all-targets" } }} --features "{{ version }},{{ features }}" --jobs 1 {{ if check == "true" { "" } else { "-- -Dwarnings" } }}

# Run tests for a single matter-spec version
[group('matter-spec')]
spec-test version=latest features=test_feat:
    cargo test -p matter-spec --locked --features "{{ version }},{{ features }}" --jobs 1

# Lint all matter-spec versions sequentially
[group('matter-spec')]
spec-clippy-all: (_each-spec "spec-clippy")

# Run tests across all matter-spec versions sequentially
[group('matter-spec')]
spec-test-all: (_each-spec "spec-test")

# Build matter-spec documentation with warnings denied and run doc-tests (default: latest)
[group('matter-spec')]
spec-doc version=latest features=feat:
    RUSTDOCFLAGS="-D warnings" cargo doc -p matter-spec --locked --jobs 1 --no-deps --features "{{ version }},{{ features }}"
    cargo test -p matter-spec --locked --jobs 1 --doc --features "{{ version }},{{ features }}"

# -----------------------------------------------------------------------------
# xtask commands
# -----------------------------------------------------------------------------

# Autogenerate matter-spec cluster models from XML specifications
[arg('fetch', long, value='true')]
[arg('force', long, value='true')]
[arg('pin', long)]
[group('xtask')]
spec-codegen fetch="false" pin="" force="false":
    cargo xtask codegen-spec {{ if fetch == "true" { "--fetch" } else { "" } }} {{ if force == "true" { "--force" } else { "" } }} {{ if pin != "" { "--pin \"" + pin + "\"" } else { "" } }}

# List cached (or remote upstream) Matter specification versions
[arg('fetch', long, value='true')]
[group('xtask')]
spec-list fetch="false":
    cargo xtask codegen-spec --list {{ if fetch == "true" { "--fetch" } else { "" } }}

# -----------------------------------------------------------------------------
# Internal helpers (located at the end)
# -----------------------------------------------------------------------------

# Internal helper to ensure a required tool is installed
[private]
[unix]
_require check hint:
    @{{ check }} >/dev/null 2>&1 || { echo "error: missing required tool. Run: {{ hint }}"; exit 1; }

[private]
[windows]
_require check hint:
    @{{ check }} *> $null; if ($LASTEXITCODE -ne 0) { Write-Error "missing required tool. Run: {{ hint }}"; exit 1 }

# Internal helper to discover and list all available matter-spec versions on Unix
[private]
[unix]
_list-specs:
    @for d in spec/matter-spec/src/v[0-9]*; do test -d "$d" && basename "$d"; done

# Internal helper to discover and list all available matter-spec versions on Windows
[private]
[windows]
_list-specs:
    @(Get-ChildItem -Directory -Path "spec/matter-spec/src/v[0-9]*").Name

# Internal helper to execute a recipe for each matter-spec version sequentially on Unix
[private]
[unix]
_each-spec recipe:
    @for v in $(just _list-specs); do \
        echo "==> Running {{ recipe }} for $v..."; \
        just {{ recipe }} "$v" || exit 1; \
    done; \
    echo "==> Successfully completed {{ recipe }} for all matter-spec versions."

# Internal helper to execute a recipe for each matter-spec version sequentially on Windows
[private]
[windows]
_each-spec recipe:
    @foreach ($v in (just _list-specs)) { \
        Write-Host "==> Running {{ recipe }} for $v..."; \
        just {{ recipe }} $v; \
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }; \
    }; \
    Write-Host "==> Successfully completed {{ recipe }} for all matter-spec versions."

# Internal dependency checker helper for Unix (macOS / Linux / WSL / Git Bash)
# Discovers all Cargo.toml files in the repository excluding target/ without git dependency.
[private]
[unix]
_check-deps:
    @failed=0; \
    for toml in $(find . -name target -prune -o -name Cargo.toml -print); do \
        test -f "$toml" || continue; \
        ws_err=$(awk ' \
            /^\[workspace\.dependencies\]/{ws=1;next} \
            /^\[/{ws=0} \
            ws && /^[ \t]*[a-zA-Z0-9_-]+[ \t]*=[ \t]*"[^"]*"/{print FILENAME ":" NR ": string shorthand forbidden under [workspace.dependencies]: " $0} \
            ws && /^[ \t]*[a-zA-Z0-9_-]+[ \t]*=.*[^-]features[ \t]*=/{print FILENAME ":" NR ": features forbidden under [workspace.dependencies]: " $0} \
        ' "$toml"); \
        if [ -n "$ws_err" ]; then \
            echo "$ws_err"; \
            failed=1; \
        fi; \
        dep_err=$(awk ' \
            /^\[/{ \
                t=$0; gsub(/^[ \t]*\[+|\]+.*$/, "", t); \
                deps=(t ~ /^((dev-|build-)?dependencies|target\..+\.(dev-|build-)?dependencies)(\.[a-zA-Z0-9_-]+)?$/); \
                subtable=(t ~ /^((dev-|build-)?dependencies|target\..+\.(dev-|build-)?dependencies)\.[a-zA-Z0-9_-]+$/); \
                next \
            } \
            deps && /^[ \t]*[a-zA-Z0-9_-]+\.workspace[ \t]*=/{print FILENAME ":" NR ": dotted dependency key forbidden (use { workspace = true }): " $0} \
            deps && !subtable && /^[ \t]*[a-zA-Z0-9_-]+[ \t]*=[ \t]*"[^"]*"/{print FILENAME ":" NR ": string shorthand forbidden in member dependencies: " $0} \
            deps && /^[ \t]*(version[ \t]*=|[a-zA-Z0-9_-]+[ \t]*=.*version[ \t]*=)/{print FILENAME ":" NR ": standalone version forbidden in member dependencies (inherit from workspace): " $0} \
        ' "$toml"); \
        if [ -n "$dep_err" ]; then \
            echo "$dep_err"; \
            failed=1; \
        fi; \
    done; \
    if [ $failed -ne 0 ]; then exit 1; fi; \
    echo "==> Cargo.toml dependency checks passed."

# Internal dependency checker helper for Windows (native PowerShell)
# Discovers all Cargo.toml files in the repository excluding target/ without git dependency.
[private]
[windows]
_check-deps:
    @$failed = $false; \
    $files = Get-ChildItem -Path . -Recurse -Filter 'Cargo.toml' | Where-Object { $_.FullName -notmatch '[\\/]target[\\/]' }; \
    foreach ($fileObj in $files) { \
        $file = $fileObj.FullName; \
        $inWs = $false; \
        $inDeps = $false; \
        $inSubtable = $false; \
        $lineNo = 0; \
        Get-Content $file | ForEach-Object { \
            $lineNo++; \
            $line = $_.Trim(); \
            if ($line -match '^\[workspace\.dependencies\]') { $inWs = $true; $inDeps = $false; $inSubtable = $false; return }; \
            if ($line -match '^\[') { \
                $inWs = $false; \
                $inDeps = ($line -match '^\[((dev-|build-)?dependencies|target\..+\.(dev-|build-)?dependencies)(\.[a-zA-Z0-9_-]+)?\]'); \
                $inSubtable = ($line -match '^\[((dev-|build-)?dependencies|target\..+\.(dev-|build-)?dependencies)\.[a-zA-Z0-9_-]+\]'); \
                return; \
            }; \
            if ($inWs -and ($line -match '^[a-zA-Z0-9_-]+\s*=\s*"[^"]*"')) { \
                Write-Error "${file}:${lineNo}: String shorthand forbidden under [workspace.dependencies]: $line"; \
                $failed = $true; \
            }; \
            if ($inWs -and ($line -match '^[a-zA-Z0-9_-]+\s*=.*[^-]features\s*=')) { \
                Write-Error "${file}:${lineNo}: Features forbidden under [workspace.dependencies]: $line"; \
                $failed = $true; \
            }; \
            if ($inDeps -and ($line -match '^[a-zA-Z0-9_-]+\.workspace\s*=')) { \
                Write-Error "${file}:${lineNo}: Dotted dependency key forbidden: $line. Use object shape { workspace = true }"; \
                $failed = $true; \
            }; \
            if ($inDeps -and (-not $inSubtable) -and ($line -match '^[a-zA-Z0-9_-]+\s*=\s*"[^"]*"')) { \
                Write-Error "${file}:${lineNo}: String shorthand forbidden in member dependencies: $line"; \
                $failed = $true; \
            }; \
            if ($inDeps -and ($line -match '^(version\s*=|[a-zA-Z0-9_-]+\s*=.*version\s*=)')) { \
                Write-Error "${file}:${lineNo}: Standalone version forbidden in member dependencies: $line"; \
                $failed = $true; \
            }; \
        }; \
    }; \
    if ($failed) { exit 1 } else { Write-Host '==> Cargo.toml dependency checks passed.' }

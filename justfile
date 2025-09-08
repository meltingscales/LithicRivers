# Cross-platform justfile for LithicRivers
# Works on Linux and MacOS.

# Default toolchain
toolchain := env_var_or_default("TOOLCHAIN", "nightly")

# Cross-platform CPU count detection
cpu_count := `nproc 2>/dev/null || echo 4`

# Rust toolchain helpers
run_cmd := "rustup run " + toolchain
# Get path to rustc for the current toolchain
rustc_bin := `which rustc 2>/dev/null || echo rustc`

# Common flags/env
lockfile_flag := "-Z next-lockfile-bump"
rustflags_common := "-Z unstable-options"
bootstrap := "1"

# Cargo helpers
cargo_base := run_cmd + " cargo"
# Ensure rustdoc also receives -Z unstable-options so that Cargo's --check-cfg flags work in doctests
rustdocflags_common := "-Z unstable-options"
cargo_env := "RUSTFLAGS='" + rustflags_common + "' RUSTDOCFLAGS='" + rustdocflags_common + "' RUSTC_BOOTSTRAP=" + bootstrap + " " + cargo_base
cargoz := cargo_base + " " + lockfile_flag
cargoz_env := "RUSTFLAGS='" + rustflags_common + "' RUSTDOCFLAGS='" + rustdocflags_common + "' RUSTC_BOOTSTRAP=" + bootstrap + " " + cargo_base + " " + lockfile_flag

build_flags := "-j " + cpu_count

# Default recipe (shows help)
default:
    @just --list

# Install dependencies
install:
    rustup toolchain install {{toolchain}}
    rustup override set {{toolchain}}
    rustup default {{toolchain}}

# Run security audit
security:
    {{cargoz_env}} audit

test: build
    {{cargoz_env}} test

test-release: build-release
    {{cargoz_env}} test --release

clean:
    rm -rf target/debug/config/
    rm -rf target/release/config/
    rm -rf artifacts/
    rm -f LithicRivers.log.*
    rm -f perf.data
    rm -f perf.data.old
    rm -f flamegraph.svg
    rm -rf steampipe_out/
    rm -f *.speedscope
    rm -f profile_cpu.svg
    rm -f security-report.json
    rm -f security-report.html
    rm -f code-analysis.json
    rm -f modules.svg
    rm -f client-deps.svg
    rm -f core-deps.svg
    rm -f client-structure.txt
    rm -f core-structure.txt
    rm -f client-deps.txt
    rm -f core-deps.txt
    rm -f code-analysis.json

git-data:
    git describe --tags --abbrev=0 > VERSION
    git rev-parse HEAD > GIT_SHA
    git rev-parse --abbrev-ref HEAD > GIT_BRANCH

copy-config-data:
    cp -f CHANGELOG.txt crates/client/assets/config/CHANGELOG.txt
    cp -f STEAM_APP_ID crates/client/assets/config/STEAM_APP_ID
    cp -f VERSION crates/client/assets/config/VERSION
    cp -f GIT_SHA crates/client/assets/config/GIT_SHA
    cp -f GIT_BRANCH crates/client/assets/config/GIT_BRANCH
    cp -f LICENSE crates/client/assets/config/LICENSE
    cp -f THIRD-PARTY-NOTICES.txt crates/client/assets/config/THIRD-PARTY-NOTICES.txt

# Build the project
build: fmt clean git-data copy-config-data
    {{cargo_base}} --version
    # Build only the stable targets to keep `just build` green
    {{cargoz_env}} build -p lithicrivers-core {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client {{build_flags}}

# Build the project in release mode
build-release: fmt clean git-data copy-config-data build-demos-release
    {{cargo_base}} --version
    {{cargoz_env}} build -p lithicrivers-core --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

build-release-no-clean: fmt git-data copy-config-data build-demos-release
    {{cargo_base}} --version
    {{cargoz_env}} build -p lithicrivers-core --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

stage-artifacts-legal:
    mkdir -p artifacts/
    cp -f CHANGELOG.txt artifacts/
    cp -f LICENSE artifacts/
    cp -f THIRD-PARTY-NOTICES.txt artifacts/

stage-artifacts: build build-demos stage-artifacts-legal
    rm -rf artifacts/
    mkdir -p artifacts/
    cp -f target/debug/lithicrivers-client artifacts/
    cp -f target/debug/demo_* artifacts/
    cp -f scripts/launcher/lithicrivers-launcher.sh artifacts/

# Stage release artifacts
stage-artifacts-release: build-release build-demos-release stage-artifacts-legal
    rm -rf artifacts/
    mkdir -p artifacts/
    cp -f target/release/lithicrivers-client artifacts/
    cp -f target/release/demo_* artifacts/
    cp -f scripts/launcher/lithicrivers-launcher.sh artifacts/

## Optional: build demo binaries (may require ratatui API updates)
build-demos: fmt
    {{cargoz_env}} build -p lithicrivers-client --bin demo_inventory {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_crafting {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_body {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_color_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_sprite_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_portrait_sprite_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_combat_chrono_trigger {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_dungeon_generation {{build_flags}}

# Optional: build demo binaries (release)
build-demos-release: fmt
    {{cargoz_env}} build -p lithicrivers-client --bin demo_inventory --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_crafting --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_body --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_color_test --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_sprite_test --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_portrait_sprite_test --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_combat_chrono_trigger --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_dungeon_generation --release {{build_flags}}

# Run debug build (alias for client)
run-debug: client

# Run the client
client: build
    {{cargoz_env}} run -p lithicrivers-client --bin lithicrivers-client

# Run release build
client-release: build-release
    {{cargoz_env}} run -p lithicrivers-client --bin lithicrivers-client --release

# Run the Intro demo
demo-intro:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_intro

# Run the Inventory UI demo
demo-inventory:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_inventory

# Run the Crafting UI demo
demo-crafting:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_crafting

# Run the Sprite Test demo
demo-sprite-test:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_beezzaroll_sprite_test

# Run the Color Test demo
demo-color-test:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_beezzaroll_color_test

# Run the portrait sprite test
demo-portrait-sprite-test:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_portrait_sprite_test

# Run the Body/Repair UI demo
demo-body:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_body

# Run the Chrono Trigger combat demo
demo-combat-chrono-trigger:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_combat_chrono_trigger

# Run the dungeon generation demo
demo-dungeon-generation:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_dungeon_generation

# Blind mode (not implemented)
client-blind:
    @echo "Blind mode not implemented yet."
    @exit 1

# Format code
fmt:
    {{cargo_env}} fmt --all

# Run clippy
clippy:
    {{cargoz_env}} clippy --all-targets --all-features {{build_flags}} -- -D warnings

# tokei, code stats
tokei:
    rustup run {{toolchain}} cargo install tokei --locked
    rustup run {{toolchain}} tokei --sort lines --type rust
    rustup run {{toolchain}} tokei --files --sort lines --type rust

# Analyze module structure and dependencies
analyze-modules:
    {{cargo_base}} install cargo-modules --locked
    @echo "=== Core crate structure ==="
    {{cargo_base}} modules structure -p lithicrivers-core --lib
    {{cargo_base}} modules structure -p lithicrivers-core --lib > core-structure.txt
    @echo ""
    @echo "=== Client main binary structure ==="
    {{cargo_base}} modules structure -p lithicrivers-client --bin lithicrivers-client
    {{cargo_base}} modules structure -p lithicrivers-client --bin lithicrivers-client > client-structure.txt
    @echo "Use 'just analyze-deps' for dependency graph"

# Analyze module dependencies as graph
analyze-deps:
    {{cargo_base}} install cargo-modules --locked

    {{cargo_base}} modules dependencies -p lithicrivers-core --lib | dot -Tsvg > core-deps.svg
    {{cargo_base}} modules dependencies -p lithicrivers-core --lib > core-deps.txt

    {{cargo_base}} modules dependencies -p lithicrivers-client --bin lithicrivers-client | dot -Tsvg > client-deps.svg
    {{cargo_base}} modules dependencies -p lithicrivers-client --bin lithicrivers-client > client-deps.txt
    @echo "Dependency graphs saved to core-deps.svg and client-deps.svg"

# Analyze code complexity using clippy
complexity:
    {{cargoz_env}} clippy --all-targets --all-features -- -W clippy::cognitive_complexity -W clippy::cyclomatic_complexity

# Analyze code complexity and metrics (DEPRECATED - use 'complexity' instead)
analyze-code:
    @echo "WARNING: 'analyze-code' is deprecated. Use 'just complexity' for better output."
    {{cargo_base}} install rust-code-analysis-cli --locked
    rm -rf code-analysis/
    mkdir -p code-analysis
    ~/.cargo/bin/rust-code-analysis-cli -p crates/ --metrics -O json -o code-analysis/
    @echo "Code analysis saved to code-analysis/ directory"

# Show toolchain information
toolchain:
    rustup show

# RenderDoc GPU capture
renderdoc:
    @echo "Launching client under RenderDoc. Close app to finish capture."
    renderdoccmd capture -- {{cargoz_env}} run -p lithicrivers-client --release

# --------------------
# Profiling (Flamegraph)
# --------------------

# Install tooling required for cargo-flamegraph
profile-setup:
    cargo install flamegraph --version 0.6.5 --locked
    cargo install inferno --locked
    echo "Make sure 'perf' is installed. You may just want to compile it from source."
    sudo sysctl kernel.perf_event_paranoid=1
    sudo sysctl kernel.kptr_restrict=0

# Run the client under cargo-flamegraph. Interact, then quit with 'q'.
profile-client: clean build
    perf record -F 500 --call-graph fp -- target/debug/lithicrivers-client
    @echo "Flamegraph written to flamegraph.svg (and perf.data)."
    perf script -i perf.data > profile.linux-perf.txt
    @echo "You can upload profile.linux-perf.txt to https://speedscope.app/"

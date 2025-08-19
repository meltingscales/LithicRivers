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
    @just _install-dev-deps

# Run security audit
security:
    {{cargoz_env}} audit

# Run tests
test:
    {{cargoz_env}} test

clean:
    rm -f target/debug/lithicrivers-client

# Build the project
build: fmt
    cp -f STEAM_APP_ID crates/client/assets/config/STEAM_APP_ID
    cp -f VERSION crates/client/assets/config/VERSION
    cp -f LICENSE crates/client/assets/config/LICENSE
    cp -f THIRD-PARTY-NOTICES.txt crates/client/assets/config/THIRD-PARTY-NOTICES.txt
    {{cargo_base}} --version
    # Build only the stable targets to keep `just build` green
    {{cargoz_env}} build -p lithicrivers-core {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client {{build_flags}}

stage-artifacts: build build-demos
    rm -rf artifacts/
    mkdir -p artifacts/
    cp -f target/debug/lithicrivers-client artifacts/
    cp -f target/debug/demo_inventory artifacts/
    cp -f target/debug/demo_body artifacts/
    cp -f target/debug/demo_inventory artifacts/
    cp -f target/debug/beezzaroll_color_test artifacts/
    cp -f target/debug/beezzaroll_sprite_test artifacts/

## Optional: build demo binaries (may require ratatui API updates)
build-demos: fmt
    {{cargoz_env}} build -p lithicrivers-client --bin demo_inventory {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_body {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin beezzaroll_color_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin beezzaroll_sprite_test {{build_flags}}

# Run debug build (alias for client)
run-debug: client

# Run the client
client:
    {{cargoz_env}} run -p lithicrivers-client --bin lithicrivers-client

# Run release build
run-release:
    {{cargoz_env}} run -p lithicrivers-client --bin lithicrivers-client --release

# Run the Intro demo
demo-intro:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_intro

# Run the Inventory UI demo
demo-inventory:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_inventory

# Run the Sprite Test demo
demo-sprite-test:
    {{cargoz_env}} run -p lithicrivers-client --bin beezzaroll_sprite_test

# Run the Color Test demo
demo-color-test:
    {{cargoz_env}} run -p lithicrivers-client --bin beezzaroll_color_test

# Run the Body/Repair UI demo
demo-body:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_body

# Blind mode (not implemented)
client-blind:
    @echo "Blind mode not implemented yet."
    @exit 1

# Format code
fmt:
    {{cargo_env}} fmt --all

# Run clippy
clippy:
    {{cargoz_env}} clippy --all-targets --all-features -D warnings

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

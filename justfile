# Cross-platform justfile for LithicRivers
# Works on Linux, Windows, and macOS

# Set shell for Windows (assumes MinGW/Git Bash is available)
set windows-shell := ["sh", "-c"]

# Default toolchain
toolchain := env_var_or_default("TOOLCHAIN", "nightly")

# Cross-platform CPU count detection
cpu_count := if os() == "windows" { 
    env_var_or_default("NUMBER_OF_PROCESSORS", "4") 
} else { 
    `nproc 2>/dev/null || echo 4` 
}

# Rust toolchain helpers
run_cmd := "rustup run " + toolchain
# Get path to rustc for the current toolchain
rustc_bin := if os() == "windows" {
    `where rustc 2>nul || echo rustc`
} else {
    `which rustc 2>/dev/null || echo rustc`
}

# Common flags/env
lockfile_flag := "-Z next-lockfile-bump"
rustflags_common := "-Z unstable-options"
bootstrap := "1"

# Cargo helpers
cargo_base := run_cmd + " cargo"
cargo_env := "RUSTFLAGS='" + rustflags_common + "' RUSTC_BOOTSTRAP=" + bootstrap + " " + cargo_base
cargoz := cargo_base + " " + lockfile_flag
cargoz_env := "RUSTFLAGS='" + rustflags_common + "' RUSTC_BOOTSTRAP=" + bootstrap + " " + cargo_base + " " + lockfile_flag

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

# Cross-platform dev dependencies installation
_install-dev-deps:
    #!/usr/bin/env sh
    if [ "{{os()}}" = "windows" ]; then
        if [ -f "scripts/install_dev_deps.cmd" ]; then
            ./scripts/install_dev_deps.cmd
        else
            echo "Warning: scripts/install_dev_deps.cmd not found"
        fi
    else
        if [ -f "scripts/install_dev_deps.sh" ]; then
            ./scripts/install_dev_deps.sh
        else
            echo "Warning: scripts/install_dev_deps.sh not found"
        fi
    fi

# Run security audit
security:
    {{cargoz_env}} audit

# Run tests
test:
    {{cargoz_env}} test

# Build the project
build:
    {{cargo_base}} --version
    # Build only the stable targets to keep `just build` green
    {{cargoz_env}} build -p lithicrivers-core {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client {{build_flags}}

## Optional: build demo binaries (may require ratatui API updates)
build-demos:
    {{cargoz_env}} build -p lithicrivers-client --bin demo_inventory {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_body {{build_flags}}

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

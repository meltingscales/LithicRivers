# Cross-platform justfile for LithicRivers
# Works on Linux and MacOS.

# Use PowerShell on Windows
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Use Bash on everything else (default)
set shell := ["bash", "-c"]

toolchain := env_var_or_default("TOOLCHAIN", "nightly")
build_flags := ""

# Default recipe (shows help)
default:
    @just --list

# Release a version to Steam and GitHub Releases
util-publish-release-windows version: test-release-windows
    powershell scripts/util-publish-release.ps1 {{version}}

# Release a version to Steam and GitHub Releases
util-publish-release-linux version: test-release-linux
    powershell scripts/util-publish-release.sh {{version}}

# Install dependencies
install:
    rustup toolchain install {{toolchain}}
    rustup override set {{toolchain}}
    rustup default {{toolchain}}
    @echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --locked
    @echo "Installing cargo-audit..."
    cargo install cargo-audit --locked


# Run security audit on dependencies
code-security:
    @echo "Running cargo-audit to check for security vulnerabilities..."
    cargo audit

test-windows: build-windows
    cargo test

test-linux: build-linux
    cargo test

test-release-windows: build-release-windows
    cargo test --release

test-release-linux: build-release-linux
    cargo test --release

# Run tests with coverage report using cargo-tarpaulin
code-coverage:
    @echo "Running tests with coverage..."
    # --verbose
    cargo tarpaulin --skip-clean --all-features --workspace --timeout 120 --out Html --out Xml --output-dir coverage/
    @echo "Coverage report generated in coverage/ directory"
    @echo "Open coverage/tarpaulin-report.html in your browser to view the report"

clean-windows:
    powershell.exe scripts/clean.ps1

clean-linux:
    bash scripts/clean.sh

git-data:
    git describe --tags --always --abbrev=0 > VERSION
    git rev-parse HEAD > GIT_SHA
    git rev-parse --abbrev-ref HEAD > GIT_BRANCH

copy-config-data-windows:
    powershell.exe scripts/copy-config-data.ps1

copy-config-data-linux:
    bash scripts/copy-config-data.sh

# Build the project
build-windows: code-fmt util-voxelbuilder-import clean-windows git-data copy-config-data-windows
    cargo --version
    # Build only the stable targets to keep `just build` green
    cargo build -p lithicrivers-core {{build_flags}}
    cargo build -p lithicrivers-client --bin lithicrivers-client {{build_flags}}

# Build the project
build-linux: code-fmt util-voxelbuilder-import clean-linux git-data copy-config-data-linux
    cargo --version
    # Build only the stable targets to keep `just build` green
    cargo build -p lithicrivers-core {{build_flags}}
    cargo build -p lithicrivers-client --bin lithicrivers-client {{build_flags}}

# Build the project in release mode
build-release-windows: code-fmt util-voxelbuilder-import-release clean-windows git-data copy-config-data-windows build-demos-release
    cargo --version
    cargo build -p lithicrivers-core --release {{build_flags}}
    cargo build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

# Build the project in release mode
build-release-linux: code-fmt util-voxelbuilder-import-release clean-linux git-data copy-config-data-linux build-demos-release
    cargo --version
    cargo build -p lithicrivers-core --release {{build_flags}}
    cargo build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

build-release-no-clean-windows: code-fmt util-voxelbuilder-import-release git-data copy-config-data-windows build-demos-release
    cargo --version
    cargo build -p lithicrivers-core --release {{build_flags}}
    cargo build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

build-release-no-clean-linux: code-fmt util-voxelbuilder-import-release git-data copy-config-data-linux build-demos-release
    cargo --version
    cargo build -p lithicrivers-core --release {{build_flags}}
    cargo build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

stage-artifacts-legal-linux:
    bash scripts/stage-artifacts-legal.sh

stage-artifacts-legal-windows:
    powershell.exe scripts/stage-artifacts-legal.ps1

# Clean artifacts directory
clean-artifacts-linux:
    bash scripts/clean-artifacts.sh

# Clean artifacts directory
clean-artifacts-windows:
    powershell.exe scripts/clean-artifacts.ps1

stage-artifacts-linux: clean-artifacts-linux build-linux build-demos stage-artifacts-legal-linux
    bash scripts/stage-artifacts.sh

stage-artifacts-windows: clean-artifacts-windows build-windows build-demos stage-artifacts-legal-windows
    bash scripts/stage-artifacts.sh

# Stage release artifacts
stage-artifacts-release-linux: clean-artifacts-linux build-release-linux build-demos-release stage-artifacts-legal-linux
    bash scripts/stage-artifacts-release.sh

# Stage release artifacts
stage-artifacts-release-windows: clean-artifacts-windows build-release-windows build-demos-release stage-artifacts-legal-windows
    bash scripts/stage-artifacts-release.sh

## Optional: build demo binaries (may require ratatui API updates)
build-demos: code-fmt util-voxelbuilder-import
    cargo build -p lithicrivers-client --bin demo_inventory {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_crafting {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_body {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_beezzaroll_color_test {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_beezzaroll_sprite_test {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_portrait_sprite_test {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_combat_chrono_trigger {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_dungeon_generation {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_corpse_looting {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_dialogue_interactions {{build_flags}}
    cargo build -p lithicrivers-client --bin utility_voxelbuilder_importer {{build_flags}}
    cargo build -p lithicrivers-client --bin utility_sprite_demo {{build_flags}}

# Optional: build demo binaries (release)
build-demos-release: code-fmt util-voxelbuilder-import
    cargo build -p lithicrivers-client --bin demo_inventory --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_crafting --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_body --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_beezzaroll_color_test --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_beezzaroll_sprite_test --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_portrait_sprite_test --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_combat_chrono_trigger --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_dungeon_generation --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_corpse_looting --release {{build_flags}}
    cargo build -p lithicrivers-client --bin demo_dialogue_interactions --release {{build_flags}}
    cargo build -p lithicrivers-client --bin utility_sprite_demo --release {{build_flags}}
    cargo build -p lithicrivers-client --bin utility_voxelbuilder_importer --release {{build_flags}}

# Run the client
client-windows: build-windows
    cargo run -p lithicrivers-client --bin lithicrivers-client

# Run the client
client-linux: build-linux
    cargo run -p lithicrivers-client --bin lithicrivers-client

# Run release build
client-release-windows: build-release-windows
    cargo run -p lithicrivers-client --bin lithicrivers-client --release

# Run release build
client-release-linux: build-release-linux
    cargo run -p lithicrivers-client --bin lithicrivers-client --release

# Run the Intro demo
demo-intro:
    cargo run -p lithicrivers-client --bin demo_intro

# Run the Inventory UI demo
demo-inventory:
    cargo run -p lithicrivers-client --bin demo_inventory

# Run the Crafting UI demo
demo-crafting:
    cargo run -p lithicrivers-client --bin demo_crafting

# Run the Sprite Test demo
demo-sprite-test:
    cargo run -p lithicrivers-client --bin demo_beezzaroll_sprite_test

# Run the Color Test demo
demo-color-test:
    cargo run -p lithicrivers-client --bin demo_beezzaroll_color_test

# Run the portrait sprite test
demo-portrait-sprite-test:
    cargo run -p lithicrivers-client --bin demo_portrait_sprite_test

# Run the Body/Repair UI demo
demo-body:
    cargo run -p lithicrivers-client --bin demo_body

# Run the Chrono Trigger combat demo
demo-combat-chrono-trigger:
    cargo run -p lithicrivers-client --bin demo_combat_chrono_trigger

# Run the dungeon generation demo
demo-dungeon-generation:
    cargo run -p lithicrivers-client --bin demo_dungeon_generation

# Run the corpse looting demo
demo-corpse-looting:
    cargo run -p lithicrivers-client --bin demo_corpse_looting

# Run the dialogue interactions demo
demo-dialogue-interactions:
    cargo run -p lithicrivers-client --bin demo_dialogue_interactions

# Run a sprite demo that showcases sprite animations and interactions
util-sprite-demo:
    cargo run -p lithicrivers-client --bin utility_sprite_demo

# Import Voxel Builder JSON files to LithicRivers .lrstructure format
util-voxelbuilder-import:
    @echo "🧊 Running Voxel Builder Structure Importer..."
    cargo run -p lithicrivers-client --bin utility_voxelbuilder_importer

util-voxelbuilder-import-release:
    @echo "🧊 Running Voxel Builder Structure Importer..."
    cargo run -p lithicrivers-client --bin utility_voxelbuilder_importer --release

util-voxelbuilder-import-site:
    @echo "Visit https://nimadez.github.io/voxel-builder/ to create voxel models."
    @echo "Export as JSON format and add to input_output_map in utility_voxelbuilder_importer.rs."
    @echo "Then run 'just util-voxelbuilder-import' to import the structure."

util-monitor-logs:
    @echo "Monitoring logs..."
    @tail -f LithicRivers.log.`date +%Y-%m-%d`

# Blind mode (not implemented)
client-blind:
    @echo "Blind mode not implemented yet."
    @exit 1

# Format code
code-fmt:
    cargo fmt --all

# Run clippy
code-clippy:
    cargo clippy --all-targets --all-features {{build_flags}} -- -D warnings

# tokei, code stats
code-tokei:
    rustup run {{toolchain}} cargo install tokei --locked
    rustup run {{toolchain}} tokei --sort lines --type rust
    rustup run {{toolchain}} tokei --files --sort lines --type rust

# Analyze module structure and dependencies
code-analyze-modules:
    cargo install cargo-modules --locked
    @echo "=== Core crate structure ==="
    cargo modules structure -p lithicrivers-core --lib
    cargo modules structure -p lithicrivers-core --lib > core-structure.txt
    @echo ""
    @echo "=== Client main binary structure ==="
    cargo modules structure -p lithicrivers-client --bin lithicrivers-client
    cargo modules structure -p lithicrivers-client --bin lithicrivers-client > client-structure.txt
    @echo "Use 'just analyze-deps' for dependency graph"

# Analyze module dependencies as graph
code-analyze-deps:
    cargo install cargo-modules --locked

    cargo modules dependencies -p lithicrivers-core --lib | dot -Tsvg > core-deps.svg
    cargo modules dependencies -p lithicrivers-core --lib > core-deps.txt

    cargo modules dependencies -p lithicrivers-client --bin lithicrivers-client | dot -Tsvg > client-deps.svg
    cargo modules dependencies -p lithicrivers-client --bin lithicrivers-client > client-deps.txt
    @echo "Dependency graphs saved to core-deps.svg and client-deps.svg"

# Analyze code complexity using clippy
code-complexity:
    cargo clippy --all-targets --all-features -- -W clippy::cognitive_complexity -W clippy::cyclomatic_complexity

# Show toolchain information
toolchain:
    rustup show

# RenderDoc GPU capture
renderdoc:
    @echo "Launching client under RenderDoc. Close app to finish capture."
    renderdoccmd capture -- cargo run -p lithicrivers-client --release

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
profile-client-windows: clean-windows build-windows
    perf record -F 500 --call-graph fp -- target/debug/lithicrivers-client
    @echo "Flamegraph written to flamegraph.svg (and perf.data)."
    perf script -i perf.data > profile.linux-perf.txt
    @echo "You can upload profile.linux-perf.txt to https://speedscope.app/"

# Run the client under cargo-flamegraph. Interact, then quit with 'q'.
profile-client-linux: clean-linux build-linux
    perf record -F 500 --call-graph fp -- target/debug/lithicrivers-client
    @echo "Flamegraph written to flamegraph.svg (and perf.data)."
    perf script -i perf.data > profile.linux-perf.txt
    @echo "You can upload profile.linux-perf.txt to https://speedscope.app/"

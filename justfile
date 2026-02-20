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
rustflags_common := "-Z unstable-options -D warnings"
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

# Release a version to Steam and GitHub Releases
util-publish-release version: test-release
  #!/usr/bin/env bash
  set -euo pipefail

  # Validate version format (basic check for vX.Y.Z format)
  if [[ ! "{{version}}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
    echo "❌ Error: Version must be in format 'vX.Y.Z' (e.g. v1.0.0)"
    exit 1
  fi

  # Check if tag already exists
  if git tag -l | grep -q "^{{version}}$"; then
    echo "❌ Error: Tag '{{version}}' already exists"
    git tag -l | grep "{{version}}"
    exit 1
  fi

  # Check if working directory is clean
  if [[ -n $(git status --porcelain) ]]; then
    echo "⚠️  Working directory has uncommitted changes:"
    git status --short
    echo ""
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      echo "❌ Aborted"
      exit 1
    fi
  fi

  # Show changelog for review
  echo "📄 Current CHANGELOG.txt (first 10 lines):"
  echo "----------------------------------------"
  head CHANGELOG.txt || echo "⚠️  CHANGELOG.txt not found"
  echo "----------------------------------------"
  echo ""
  read -p "Have you reviewed and updated the CHANGELOG.txt for this release '{{version}}'? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Please update CHANGELOG.txt before releasing"
    exit 1
  fi

  # Final confirmation before publishing
  echo ""
  echo "🚀 Ready to publish release {{version}}"
  echo "📋 This will:"
  echo "   • Create commit and tag {{version}}"
  echo "   • Push to GitHub (triggering CI/CD)"
  echo "   • Build for Windows, macOS, and Linux"
  echo "   • Upload to GitHub Releases"
  echo "   • Upload to Steam via SteamPipe"
  echo ""
  read -p "Are you sure you want to trigger a build and upload {{version}} to GitHub and Steam? [y/N] " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
  fi

  echo "🚀 Publishing release {{version}}..."

  # Add all changes
  git add .

  # Create commit with version
  git commit -m "Release {{version}}" || echo "⚠️  No changes to commit"

  # Create and push tag
  git tag "{{version}}"

  echo "📤 Pushing to remote..."
  git push origin
  git push origin --tags

  echo "✅ Successfully published release {{version}}"
  echo "🔗 GitHub Actions will now build and deploy the release"
  echo "🎯 Monitor the release at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[\/:]//;s/.git$//')/releases"

# Install dependencies
install:
    rustup toolchain install {{toolchain}}
    rustup override set {{toolchain}}
    rustup default {{toolchain}}
    @echo "Installing cargo-tarpaulin..."
    {{cargo_base}} install cargo-tarpaulin --locked
    @echo "Installing cargo-audit..."
    {{cargo_base}} install cargo-audit --locked


# Run security audit on dependencies
code-security:
    @echo "Running cargo-audit to check for security vulnerabilities..."
    {{cargo_base}} audit

test: build
    {{cargoz_env}} test

test-release: build-release
    {{cargoz_env}} test --release

# Run tests with coverage report using cargo-tarpaulin
code-coverage:
    @echo "Running tests with coverage..."
    # --verbose
    {{cargoz_env}} tarpaulin --skip-clean --all-features --workspace --timeout 120 --out Html --out Xml --output-dir coverage/
    @echo "Coverage report generated in coverage/ directory"
    @echo "Open coverage/tarpaulin-report.html in your browser to view the report"

clean:
    rm -rf target/debug/config/
    rm -rf target/release/config/
    rm -rf artifacts/
    rm -f save.json
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
    rm -rf coverage/

git-data:
    git describe --tags --always --abbrev=0 > VERSION
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
build: code-fmt util-voxelbuilder-import clean git-data copy-config-data
    {{cargo_base}} --version
    # Build only the stable targets to keep `just build` green
    {{cargoz_env}} build -p lithicrivers-core {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client {{build_flags}}

# Build the project in release mode
build-release: code-fmt util-voxelbuilder-import-release clean git-data copy-config-data build-demos-release
    {{cargo_base}} --version
    {{cargoz_env}} build -p lithicrivers-core --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

build-release-no-clean: code-fmt util-voxelbuilder-import-release git-data copy-config-data build-demos-release
    {{cargo_base}} --version
    {{cargoz_env}} build -p lithicrivers-core --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin lithicrivers-client --release {{build_flags}}

stage-artifacts-legal:
    mkdir -p artifacts/
    cp -f CHANGELOG.txt artifacts/
    cp -f LICENSE artifacts/
    cp -f THIRD-PARTY-NOTICES.txt artifacts/

# Clean artifacts directory
clean-artifacts:
    rm -rf artifacts/
    mkdir -p artifacts/

stage-artifacts: clean-artifacts build build-demos stage-artifacts-legal
    cp -f target/debug/lithicrivers-client artifacts/
    cp -f target/debug/demo_* artifacts/
    cp -f target/debug/utility_* artifacts/
    cp -f scripts/launcher/lithicrivers-launcher.sh artifacts/

# Stage release artifacts
stage-artifacts-release: clean-artifacts build-release build-demos-release stage-artifacts-legal
    cp -f target/release/lithicrivers-client artifacts/
    cp -f target/release/demo_* artifacts/
    cp -f target/release/utility_* artifacts/
    cp -f scripts/launcher/lithicrivers-launcher.sh artifacts/

## Optional: build demo binaries (may require ratatui API updates)
build-demos: code-fmt util-voxelbuilder-import
    {{cargoz_env}} build -p lithicrivers-client --bin demo_inventory {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_crafting {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_body {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_color_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_sprite_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_portrait_sprite_test {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_combat_chrono_trigger {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_dungeon_generation {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_corpse_looting {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_dialogue_interactions {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin utility_voxelbuilder_importer {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin utility_sprite_demo {{build_flags}}

# Optional: build demo binaries (release)
build-demos-release: code-fmt util-voxelbuilder-import
    {{cargoz_env}} build -p lithicrivers-client --bin demo_inventory --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_crafting --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_body --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_color_test --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_beezzaroll_sprite_test --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_portrait_sprite_test --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_combat_chrono_trigger --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_dungeon_generation --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_corpse_looting --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin demo_dialogue_interactions --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin utility_sprite_demo --release {{build_flags}}
    {{cargoz_env}} build -p lithicrivers-client --bin utility_voxelbuilder_importer --release {{build_flags}}

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

# Run the corpse looting demo
demo-corpse-looting:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_corpse_looting

# Run the dialogue interactions demo
demo-dialogue-interactions:
    {{cargoz_env}} run -p lithicrivers-client --bin demo_dialogue_interactions

# Run a sprite demo that showcases sprite animations and interactions
util-sprite-demo:
    {{cargoz_env}} run -p lithicrivers-client --bin utility_sprite_demo

# Import Voxel Builder JSON files to LithicRivers .lrstructure format
util-voxelbuilder-import:
    @echo "🧊 Running Voxel Builder Structure Importer..."
    {{cargoz_env}} run -p lithicrivers-client --bin utility_voxelbuilder_importer

util-voxelbuilder-import-release:
    @echo "🧊 Running Voxel Builder Structure Importer..."
    {{cargoz_env}} run -p lithicrivers-client --bin utility_voxelbuilder_importer --release

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
    {{cargo_env}} fmt --all

# Run clippy
code-clippy:
    {{cargoz_env}} clippy --all-targets --all-features {{build_flags}} -- -D warnings

# tokei, code stats
code-tokei:
    rustup run {{toolchain}} cargo install tokei --locked
    rustup run {{toolchain}} tokei --sort lines --type rust
    rustup run {{toolchain}} tokei --files --sort lines --type rust

# Analyze module structure and dependencies
code-analyze-modules:
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
code-analyze-deps:
    {{cargo_base}} install cargo-modules --locked

    {{cargo_base}} modules dependencies -p lithicrivers-core --lib | dot -Tsvg > core-deps.svg
    {{cargo_base}} modules dependencies -p lithicrivers-core --lib > core-deps.txt

    {{cargo_base}} modules dependencies -p lithicrivers-client --bin lithicrivers-client | dot -Tsvg > client-deps.svg
    {{cargo_base}} modules dependencies -p lithicrivers-client --bin lithicrivers-client > client-deps.txt
    @echo "Dependency graphs saved to core-deps.svg and client-deps.svg"

# Analyze code complexity using clippy
code-complexity:
    {{cargoz_env}} clippy --all-targets --all-features -- -W clippy::cognitive_complexity -W clippy::cyclomatic_complexity

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

.PHONY: help install build run-debug run-release client fmt clippy toolchain security test \
	profile-flamegraph profile-tracy renderdoc

TOOLCHAIN ?= nightly

# Helpers to avoid duplication
# RUN: invoke binaries with the pinned toolchain
RUN := rustup run $(TOOLCHAIN)
# CARGO: plain cargo under nightly
CARGO := $(RUN) cargo
# CARGO_ENV: cargo with nightly-only flags required by some deps using --check-cfg
CARGO_ENV := $(RUN) env RUSTFLAGS='-Z unstable-options' RUSTC_BOOTSTRAP=1 cargo

BUILD_FLAGS := -j `nproc`

# ---- NixOS guard ----
# If we're on NixOS and a shell.nix exists, require running inside nix-shell
# to avoid sprinkling wrapper logic in every target.
NIXOS := $(shell test -f /etc/os-release && grep -qi nixos /etc/os-release 2>/dev/null && echo 1 || echo 0)
HAVE_SHELL_NIX := $(shell test -f shell.nix && echo 1 || echo 0)
ifeq ($(NIXOS)$(HAVE_SHELL_NIX),11)
  ifeq ($(IN_NIX_SHELL),)
    $(error Detected NixOS with shell.nix. Please run 'nix-shell' before invoking make.)
  endif
endif

help:
	@echo "Note for NixOS: This Makefile requires running inside 'nix-shell' when shell.nix is present."
	@echo "Available targets:"
	@echo "BUILDING/DEV:"
	@echo "  install - Install dependencies"
	@echo "  build       - Build the project"
	@echo "  run-debug   - Run the project (debug)"
	@echo "  run-release - Run the project (release)"
	@echo "PROFILING:"
	@echo "  profile-flamegraph - CPU profile with cargo-flamegraph (perf)"
	@echo "  profile-tracy      - Run with Tracy instrumentation (requires tracy feature/deps)"
	@echo "  renderdoc          - Launch under RenderDoc (requires renderdoccmd)"
	@echo "FORMATTING/TESTING:"
	@echo "  fmt     - Format the code"
	@echo "  clippy  - Run clippy"
	@echo "  security - Run cargo audit"
	@echo "  toolchain - Show the current toolchain"
	@echo "  test    - Run tests"
	exit 0

install:
	rustup toolchain install $(TOOLCHAIN)
	rustup override set $(TOOLCHAIN)
	rustup default $(TOOLCHAIN)
	./scripts/install_dev_deps.sh

security:
	$(CARGO_ENV) audit

test:
	$(CARGO_ENV) test

build:
	$(CARGO) --version
	$(CARGO_ENV) build $(BUILD_FLAGS)
	$(CARGO_ENV) build -p lithicrivers-client $(BUILD_FLAGS)

run-debug: client

client:
	$(CARGO_ENV) run -p lithicrivers-client

run-release:
	$(CARGO_ENV) run -p lithicrivers-client --release

client-blind:
	echo "Blind mode not implemented yet."
	exit 1

fmt:
	$(CARGO_ENV) fmt --all

clippy:
	$(CARGO_ENV) clippy --all-targets --all-features -D warnings

toolchain:
	rustup show

# -------------------- Profiling helpers --------------------

# Verify external profiling tools are present. This target intentionally fails
# with clear instructions if tools are missing. It does not auto-install.
profile-deps:
	@ok=1; \
	if ! command -v cargo-flamegraph >/dev/null 2>&1; then \
	  echo "❌ Missing cargo-flamegraph. Install with: 'cargo install flamegraph --locked'"; \
	  echo "   Note: cargo-flamegraph >=0.6.8 needs rustc>=1.78. For older toolchains, try '--version 0.6.7 --locked'"; \
	  ok=0; \
	fi; \
	if ! command -v renderdoccmd >/dev/null 2>&1; then \
	  echo "⚠️  renderdoccmd not found. Install RenderDoc to use 'make renderdoc'."; \
	fi; \
	if [ $$ok -ne 1 ]; then \
	  exit 1; \
	fi

# cargo-flamegraph (Linux perf). Requires: cargo-flamegraph, perf permissions.
profile-flamegraph: profile-deps
	@echo " Running cargo-flamegraph (using perf). You may need elevated perf permissions."
	RUSTFLAGS='-Z unstable-options -g' RUSTC_BOOTSTRAP=1 $(CARGO) flamegraph -p lithicrivers-client
	RUSTC_BOOTSTRAP=1 $(CARGO) flamegraph -p lithicrivers-client --release

# Tracy live profiler. Requires: tracy viewer and enabling tracy instrumentation in code.
# By default this enables a Cargo feature named 'tracy'. Wire your client/crates to use
# bevy_tracy or tracing-tracy under this feature.
profile-tracy:
	@echo " Running client with Tracy instrumentation (feature 'tracy')."
	@echo " Launch tracy viewer with 'tracy &' before running this."
	$(CARGO_ENV) run -p lithicrivers-client --features tracy --release

# RenderDoc GPU capture. Requires: renderdoccmd available in PATH.
renderdoc: profile-deps
	@echo " Launching client under RenderDoc. Close app to finish capture."
	renderdoccmd capture -- $(CARGO_ENV) run -p lithicrivers-client --release
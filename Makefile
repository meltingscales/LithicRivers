.PHONY: help install build run-debug run-release client fmt clippy toolchain security test \
	profile-flamegraph profile-tracy renderdoc

TOOLCHAIN ?= nightly

# Helpers to avoid duplication
# Resolve the pinned toolchain and standardize env/flags in one place
# RUN: invoke binaries with the pinned toolchain
RUN := rustup run $(TOOLCHAIN)

# Export the rustc from the pinned toolchain so cargo doesn't accidentally use /usr/bin/rustc
RUSTC_BIN := $(shell rustup which --toolchain $(TOOLCHAIN) rustc)
export RUSTC := $(RUSTC_BIN)

# Common flags/env
LOCKFILE_FLAG := -Z next-lockfile-bump
RUSTFLAGS_COMMON := -Z unstable-options
BOOTSTRAP := 1

# Cargo helpers
CARGO_BASE := $(RUN) cargo
CARGO := $(CARGO_BASE)
# cargo with nightly-only flags required by some deps using --check-cfg
CARGO_ENV := RUSTFLAGS='$(RUSTFLAGS_COMMON)' RUSTC_BOOTSTRAP=$(BOOTSTRAP) $(CARGO_BASE)
# cargo with lockfile v4 parsing enabled
CARGOZ := $(CARGO_BASE) $(LOCKFILE_FLAG)
CARGOZ_ENV := RUSTFLAGS='$(RUSTFLAGS_COMMON)' RUSTC_BOOTSTRAP=$(BOOTSTRAP) $(CARGO_BASE) $(LOCKFILE_FLAG)

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
	$(CARGO) install flamegraph
	./scripts/install_dev_deps.sh

security:
	$(CARGOZ_ENV) audit

test:
	$(CARGOZ_ENV) test

build:
	$(CARGO) --version
	$(CARGOZ_ENV) build $(BUILD_FLAGS)
	$(CARGOZ_ENV) build -p lithicrivers-client $(BUILD_FLAGS)

run-debug: client

client:
	$(CARGOZ_ENV) run -p lithicrivers-client

run-release:
	$(CARGOZ_ENV) run -p lithicrivers-client --release

client-blind:
	echo "Blind mode not implemented yet."
	exit 1

fmt:
	$(CARGO_ENV) fmt --all

clippy:
	$(CARGOZ_ENV) clippy --all-targets --all-features -D warnings

toolchain:
	rustup show

# -------------------- Profiling helpers --------------------

profile-flamegraph:
	@echo " Running cargo-flamegraph (using perf). You may need elevated perf permissions."
	$(CARGOZ_ENV) flamegraph -p lithicrivers-client
	$(CARGOZ_ENV) flamegraph -p lithicrivers-client --release

# Tracy live profiler. Requires: tracy viewer and enabling tracy instrumentation in code.
# By default this enables a Cargo feature named 'tracy'. Wire your client/crates to use
# bevy_tracy or tracing-tracy under this feature.
profile-tracy:
	@echo " Running client with Tracy instrumentation (feature 'tracy')."
	@echo " Launch tracy viewer with 'tracy &' before running this."
	$(CARGOZ_ENV) run -p lithicrivers-client --features tracy --release

# RenderDoc GPU capture. Requires: renderdoccmd available in PATH.
renderdoc:
	@echo " Launching client under RenderDoc. Close app to finish capture."
	renderdoccmd capture -- $(CARGOZ_ENV) run -p lithicrivers-client --release
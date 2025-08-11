.PHONY: help install build run client fmt clippy toolchain security test

TOOLCHAIN ?= nightly

help:
	@echo "Available targets:"
	@echo "  install - Install dependencies"
	@echo "  build   - Build the project"
	@echo "  run     - Run the project"
	@echo "  fmt     - Format the code"
	@echo "  clippy  - Run clippy"
	@echo "  security - Run cargo audit"
	@echo "  toolchain - Show the current toolchain"
	@echo "  test    - Run tests"

install:
	rustup toolchain install $(TOOLCHAIN)
	rustup override set $(TOOLCHAIN)
	rustup default $(TOOLCHAIN)
	./scripts/install_dev_deps.sh

security:
	cargo audit

test:
	cargo test

build:
	cargo --version
	cargo build -j $$(nproc)
	cargo build -p lithicrivers-client -j $$(nproc)

run: client

client:
	cargo run -p lithicrivers-client

client-blind:
	echo "Blind mode not implemented yet."

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -D warnings

toolchain:
	rustup show
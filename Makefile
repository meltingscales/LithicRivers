.PHONY: help install build run client fmt clippy toolchain

TOOLCHAIN ?= 1.75.0

help:
	@echo "Available targets:"
	@echo "  install - Install dependencies"
	@echo "  build   - Build the project"
	@echo "  run     - Run the project"
	@echo "  fmt     - Format the code"
	@echo "  clippy  - Run clippy"
	@echo "  toolchain - Show the current toolchain"

install:
	rustup toolchain install $(TOOLCHAIN)
	rustup override set $(TOOLCHAIN)
	./scripts/install_dev_deps.sh

build:
	cargo --version
	cargo build

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
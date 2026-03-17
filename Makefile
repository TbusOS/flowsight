.PHONY: build test install clean completions lint fmt check help

# Default target
all: build

## Build release binary
build:
	cargo build --package flowsight-cli --release

## Build debug binary
debug:
	cargo build --package flowsight-cli

## Run workspace tests
test:
	cargo test --workspace

## Install via cargo
install:
	cargo install --path cli

## Remove build artefacts
clean:
	cargo clean

## Generate shell completions into completions/
completions: build
	@mkdir -p completions
	./target/release/flowsight completions bash > completions/flowsight.bash
	./target/release/flowsight completions zsh  > completions/_flowsight
	./target/release/flowsight completions fish > completions/flowsight.fish
	@echo "Completions written to completions/"

## Run clippy and format check
lint:
	cargo clippy --workspace -- -D warnings
	cargo fmt --all -- --check

## Format all Rust code
fmt:
	cargo fmt --all

## Type-check without building
check:
	cargo check --workspace

## Print available targets
help:
	@echo "FlowSight Makefile targets:"
	@echo ""
	@echo "  make build        Build release binary"
	@echo "  make debug        Build debug binary"
	@echo "  make test         Run workspace tests"
	@echo "  make install      Install via cargo install"
	@echo "  make clean        Remove build artefacts"
	@echo "  make completions  Generate shell completions"
	@echo "  make lint         Run clippy + format check"
	@echo "  make fmt          Format all Rust code"
	@echo "  make check        Type-check without building"
	@echo "  make help         Show this help"

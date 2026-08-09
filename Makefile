.PHONY: all build test lint fmt clean

all: build

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

clean:
	cargo clean

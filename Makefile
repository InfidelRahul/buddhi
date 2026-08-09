.PHONY: all build test lint fmt clean bench

all: build

build:
	cargo build --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

bench:
	cargo bench --workspace

clean:
	cargo clean

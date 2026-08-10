.PHONY: all build test lint fmt clean bench release

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

release:
	cargo build --release --locked

clean:
	cargo clean

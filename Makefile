RUST_SRC:=$(wildcard **/*.rs)

all: build

test: check
	@cargo test

check: fmt-check lint

build: target/release/libvst.rlib

target/release/libvst.rlib: $(RUST_SRC)
	@cargo build --release

lint: $(RUST_SRC)
	@cargo +nightly clippy

fmt: $(RUST_SRC)
	@cargo +nightly fmt

fmt-check: $(RUST_SRC)
	@cargo +nightly fmt -- --check

clean:
	@rm -fr target/

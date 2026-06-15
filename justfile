check:
    cargo check

build:
    cargo build

run: build
    cargo run

timings:
    cargo build --timings

watch:
    cargo watch -x check

test:
    cargo test

fix:
    cargo clippy --fix
    cargo fmt

build-release:
    cargo build --release

install:
    cargo install --path .

clean:
    cargo clean

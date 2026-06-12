set shell := ["powershell.exe", "-NoProfile", "-Command"]

check:
    cargo check

run:
    cargo run

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

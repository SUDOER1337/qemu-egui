# QEMU GUI

[![Rust](https://github.com/SUDOER1337/qemu-egui/actions/workflows/rust.yml/badge.svg)](https://github.com/SUDOER1337/qemu-egui/actions/workflows/rust.yml)

A minimal egui-based GUI wrapper for QEMU, designed for personal portable use from a USB drive on Windows.

## Prerequisites (local build)

- Rust toolchain: `stable-x86_64-pc-windows-gnu`
- QEMU installed and accessible at the configured path

```sh
# Install the windows-gnu toolchain
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu

# Install llvm-tools for the fast lld linker (optional, ~2-3x faster)
rustup component add llvm-tools-preview

# Install sccache compiler cache (optional, ~2-3x on re-builds)
# winget install Mozilla.sccache   # or: scoop install sccache
```

Then set these environment variables in your shell for the fastest builds:

```sh
$env:RUSTC_WRAPPER = "sccache"
```

## Build & Run

```sh
cargo run         # build and launch
cargo check       # fast compile check (no linking)
cargo clippy      # lint
cargo fmt         # format
```

Or use the `just` task runner:

```sh
just run          # cargo run
just check        # cargo check
just fix          # clippy + fmt
```


Local-only overrides (not available on CI):
 - Set `RUSTC_WRAPPER = "sccache"` in your shell for faster re-builds
 - Use `rust-lld` linker: install llvm-tools-preview via rustup, then add:
 
   ```toml
   [target.x86_64-pc-windows-gnu]
   linker = "...rustup.../bin/rust-lld.exe"
   rustflags = ["-C", "link-arg=--threads=8"]
   ```

## Usage

1. Configure paths to QEMU binary, disk image, and ISO
2. Click **Boot from Disk** or **Install (Boot from ISO)**
3. Use **Kill QEMU** to stop the VM
4. Config is auto-saved on exit or via **Save Config**

Config is persisted as `qemu-gui-config.ron` next to the binary.

## Stack

- [egui](https://github.com/emilk/egui) / eframe 0.34 (wgpu)
- serde + RON 0.8 for config persistence
- Rust 2021 edition, `x86_64-pc-windows-gnu` target

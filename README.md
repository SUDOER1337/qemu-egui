# QEMU GUI

[![Rust](https://github.com/SUDOER1337/qemu-egui/actions/workflows/rust.yml/badge.svg)](https://github.com/SUDOER1337/qemu-egui/actions/workflows/rust.yml)

A minimal egui-based GUI wrapper for QEMU, designed for personal portable use from a USB drive on Windows.

## Prerequisites

- **QEMU** installed and accessible at the configured path (e.g. `D:\qemu\qemu-system-x86_64.exe`)
- **Rust** toolchain (auto-pinned via `rust-toolchain.toml`):
  - Channel: `stable-x86_64-pc-windows-gnu`
  - Fast linker `rust-lld.exe` pre-configured in `.cargo/config.toml`

#### Optional: faster re-builds with sccache

```sh
# Install sccache (one of these):
# winget install Mozilla.sccache
# scoop install sccache

# Then in your shell before running cargo:
$env:RUSTC_WRAPPER = "sccache"
```

## Build & Run

```sh
cargo run         # build and launch
cargo check       # fast compile check (no linking)
cargo clippy      # lint
cargo fmt         # format
```

Or with `just` (PowerShell backend):

```sh
just run          # cargo run
just check        # cargo check
just fix          # clippy + fmt
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

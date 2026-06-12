# QEMU GUI

A minimal egui-based GUI wrapper for QEMU, designed for personal portable use from a USB drive on Windows.

## Build & Run

```sh
just run     # cargo run
just check   # cargo check
```

Or use cargo directly:

```sh
cargo run
```

## Usage

1. Configure paths to QEMU binary, disk image, and ISO
2. Click **Boot from Disk** or **Install (Boot from ISO)**
3. Use **Kill QEMU** to stop the VM
4. Save config with **Save Config** (persisted as `qemu-gui-config.ron`)

## Stack

- [egui](https://github.com/emilk/egui) / eframe (0.34, wgpu)
- serde + RON for config
- Rust stable (Windows GNU target)

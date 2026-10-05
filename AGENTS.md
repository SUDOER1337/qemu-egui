# QEMU GUI — Agent Notes

## Project
A minimal egui wrapper around QEMU — runs on Linux and Windows (runs from USB drive on Windows/GNU target).

## Stack
- Rust edition 2021, `stable` toolchain (resolves to native host: GNU on Windows, default on Linux)
- `eframe`/`egui` 0.34 with wgpu backend
- `serde` + `ron` 0.8 for config persistence
- `just` as task runner (powershell backend)

## Key files
- `src/main.rs` — single-file app: Config, QemuGui struct, egui App impl, main entry
- `Cargo.toml` — dependencies
- `justfile` — commands: `check`, `run`, `watch`, `test`, `fix`, `build-release`, `install`, `clean`
- `flake.nix` — Nix flake: `packages` (Linux), `devShells`, `checks`, `apps`, `formatter`
- `nix/package.nix` — `buildRustPackage` for `qemu-egui`
- `nix/clippy-check.nix` — standalone `cargo clippy --deny warnings` derivation

## Phase 1 — Done
- `Drop` impl kills QEMU child on exit (prevents orphan process)
- `child.wait()` moved to background thread (doesn't block UI)
- Config write errors are logged instead of silently ignored
- Log capped at ~10KB ring buffer (prevents unbounded growth)

## Phase 2 — Done
- Input validation before QEMU launch (empty paths, zero values)
- Config auto-saved on app exit (via `Drop`)
- Config file path shown in UI
- Standalone "Save Config" button next to path label

## Build speed — Done
- `debug = 0` in dev profile (skips debug info, faster linking)
- `codegen-units = 256` (max parallel codegen)
- `rust-lld.exe` as linker (~2-3x faster than GNU ld; configured in `.cargo/config.toml`)
- `--threads=8` in linker flags
- `sccache` as `RUSTC_WRAPPER` (opt-in via env var; documented in README)
- `rust-toolchain.toml` pins `stable` (host-native; GNU toolchain set per-machine via rustup default)
- `tools/lld-link.cmd` wraps `rust-lld.exe` via `%RUSTUP_HOME%` — no hardcoded username

## Nix flake — Done
- `flake.nix` on `nixpkgs` (nixos-unstable), systems: `x86_64-linux`, `aarch64-linux`
- `packages.qemu-egui` via `buildRustPackage`, pinned to the checked-in `Cargo.lock` (no vendor hash)
- `devShells.default`: rustc/cargo/clippy/rust-analyzer/rustfmt/cargo-watch/just/sccache/mold/qemu/gdb
- `shellHook` sets `RUSTC_WRAPPER=sccache`; `LD_LIBRARY_PATH` covers libglvnd, xkbcommon, wayland, vulkan, X11
- `checks`: `build`, `clippy` (deny warnings), `test` — all pass on `nix flake check`
- `apps.default` → `nix run`
- `overlays.default` + `formatter` (`pkgs.nixfmt`)
- Both package derivations `rm -rf .cargo tools` in `postPatch` — the repo's cargo config hardcodes the Windows lld wrapper and `-fuse-ld=mold`
- `clippy-check.nix` sets `auditable = false`; `buildRustPackage` otherwise substitutes `cargo` with the cargo-auditable wrapper, which swallows `cargo clippy`

## Known issues (current)
- Hardcoded Windows paths (intentional for personal USB-drive use)
- `accel=whpx` hardcoded (Windows-only, intentional)
- `accel=whpx` and default QEMU paths are Windows-only (intentional for USB-drive use)
- Linux build requires `mold` for the fast linker path; falls back to default `ld` if absent
- On Windows, `tools/lld-link.cmd` assumes the active toolchain is `stable-x86_64-pc-windows-gnu`
- Flake packaging is Linux-only; Windows builds still go through `just` / `cargo`
- Flake inputs come from the git tree, so new files must be `git add`ed before `nix build` sees them

## Conventions
- Single `src/main.rs` (no modules yet)
- `Config` drives all QEMU arguments
- Log uses `log_push` on `self.log: String`
- Clippy + fmt via `just fix`

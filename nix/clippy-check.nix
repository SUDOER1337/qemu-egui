# Runs `cargo clippy` inside the same dependency closure as the package build,
# so CI lint results match what a developer gets locally.
{
  lib,
  rustPlatform,
  stdenv,
  clippy,
  pkg-config,
  vulkan-loader,
  libGL,
  libxkbcommon,
  wayland,
  wayland-scanner,
  libx11,
  libxcursor,
  libxi,
  libxrandr,
  version ? "0.1.0",
}:

rustPlatform.buildRustPackage {
  pname = "qemu-egui-clippy";
  inherit version;

  src = ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    wayland-scanner
    clippy
  ];

  buildInputs = [
    libGL
    libxkbcommon
    vulkan-loader
    wayland
    libx11
    libxcursor
    libxi
    libxrandr
  ];

  # Matches the packaging build: the repo's .cargo/config.toml points the linker
  # at tools/lld-link.cmd and mold, neither of which exist in the sandbox.
  postPatch = ''
    rm -rf .cargo tools
  '';

  # buildRustPackage substitutes `cargo` with the cargo-auditable wrapper, which
  # hardcodes an `auditable` subcommand and would swallow `cargo clippy`.
  auditable = false;

  buildPhase = ''
    runHook preBuild

    cargo clippy \
      --offline \
      --all-targets \
      --all-features \
      --target ${stdenv.hostPlatform.rust.rustcTarget} \
      -- \
      --deny warnings

    touch $out
  '';

  dontInstall = true;

  meta = {
    description = "cargo clippy check for qemu-egui";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}

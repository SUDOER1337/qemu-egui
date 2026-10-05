{
  lib,
  rustPlatform,
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
  makeWrapper,
  version ? "0.1.0",
  doCheck ? false,
}:

rustPlatform.buildRustPackage {
  pname = "qemu-egui";
  inherit version doCheck;

  src = ../.;

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    wayland-scanner
    makeWrapper
  ];

  buildInputs = [
    libGL
    libxkbcommon
    vulkan-loader
    wayland

    # x11-dl / xkbcommon-dl dlopen these at runtime rather than linking,
    # but they must be on the RPATH or the binary cannot open a window.
    libx11
    libxcursor
    libxi
    libxrandr
  ];

  # The repo's .cargo/config.toml hardcodes `-fuse-ld=mold` and the Windows
  # lld wrapper, neither of which exist inside the nix sandbox. Strip it from
  # the packaged source and let stdenv pick the linker.
  postPatch = ''
    rm -rf .cargo tools
  '';

  # rfd opens files through the XDG desktop portal, which requires a session
  # bus; wgpu prefers the Vulkan loader but falls back to GL.
  postFixup = ''
    wrapProgram $out/bin/qemu_gui \
      --prefix LD_LIBRARY_PATH : ${
        lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
        ]
      }
  '';

  postInstall = ''
    ln -s qemu_gui $out/bin/qemu-egui
  '';

  meta = {
    description = "Minimal egui wrapper around QEMU";
    longDescription = ''
      A small egui/eframe front-end that builds and launches qemu-system-x86_64
      with a configurable disk image, ISO and accelerator.
    '';
    license = lib.licenses.mit;
    mainProgram = "qemu-egui";
    platforms = lib.platforms.linux;
  };
}

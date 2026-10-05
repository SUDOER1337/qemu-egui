{
  description = "Minimal egui wrapper around QEMU";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      # Windows builds go through the MSVC/GNU cross setup from the justfile,
      # so the flake only packages the Linux (and therefore CI) targets.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      version = "0.1.0";
    in
    {
      packages = forAllSystems (pkgs: rec {
        qemu-egui = pkgs.callPackage ./nix/package.nix {
          inherit version;
        };
        default = qemu-egui;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "qemu-egui";

          packages = [
            pkgs.rustc
            pkgs.cargo
            pkgs.rust-analyzer
            pkgs.rustfmt
            pkgs.clippy
            pkgs.rustup
            pkgs.cargo-watch
            pkgs.just
            pkgs.sccache
            pkgs.mold
            pkgs.qemu_kvm
            pkgs.pkg-config
            pkgs.gdb
          ];

          # eframe's wgpu backend loads these at runtime for the window/GL bits.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libglvnd
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.vulkan-loader
            pkgs.libx11
            pkgs.libxcursor
            pkgs.libxi
            pkgs.libxrandr
          ];

          shellHook = ''
            export RUSTC_WRAPPER="sccache"
          '';
        };
      });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.qemu-egui}/bin/qemu-egui";
          meta.description = "Minimal egui wrapper around QEMU";
        };
      });

      checks = forAllSystems (pkgs: {
        build = self.packages.${pkgs.stdenv.hostPlatform.system}.qemu-egui;

        clippy = pkgs.callPackage ./nix/clippy-check.nix { inherit version; };

        test = pkgs.callPackage ./nix/package.nix {
          inherit version;
          doCheck = true;
        };
      });

      overlays.default = final: prev: {
        qemu-egui = final.callPackage ./nix/package.nix { inherit version; };
      };

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}

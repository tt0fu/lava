{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        mingwW64 = pkgs.pkgsCross.mingwW64;

        packages = with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [
              "clippy"
              "rust-analyzer"
              "rust-docs"
              "rust-src"
            ];
            targets = [
              "x86_64-unknown-linux-gnu"
              "x86_64-pc-windows-gnu"
            ];
          })

          pkg-config
          makeWrapper

          alsa-lib
          alsa-utils
          pulseaudio
          pipewire
          jack2

          shaderc
          glsl_analyzer
          spirv-tools
          vulkan-loader
          vulkan-tools
          vulkan-tools-lunarg
          vulkan-validation-layers

          libGL

          libxkbcommon
          wayland
          wayland.dev
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = packages;

            nativeBuildInputs = [
              mingwW64.stdenv.cc
              mingwW64.stdenv.cc.bintools
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath packages;
            PKG_CONFIG_PATH = "${alsa-lib.dev}/lib/pkgconfig";
            SHADERC_LIB_DIR = lib.makeLibraryPath [ shaderc ];
            VK_LAYER_PATH = "${vulkan-validation-layers}/share/vulkan/explicit_layer.d";

            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${mingwW64.stdenv.cc}/bin/x86_64-w64-mingw32-gcc";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${mingwW64.windows.pthreads}/lib";

            shellHook = ''export SHELL="${pkgs.bashInteractive}/bin/bash"; '';
          };
        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "lava";
          version = "0.0.1";
          doCheck = false;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          src = ./.;
          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper

            alsa-lib
            shaderc
            jack2
          ];
          env = with pkgs; {
            PKG_CONFIG_PATH = "${alsa-lib.dev}/lib/pkgconfig:${jack2.dev}/lib/pkgconfig";
            SHADERC_LIB_DIR = lib.makeLibraryPath [ shaderc ];
          };
          postFixup = ''
            wrapProgram $out/bin/lava \
              --set LD_LIBRARY_PATH ${
                pkgs.lib.makeLibraryPath (
                  with pkgs;
                  [
                    vulkan-loader
                    vulkan-tools
                    libxkbcommon
                    wayland
                  ]
                )
              }
          '';
        };
      }
    );
}

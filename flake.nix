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
      in
      {
        devShells.default =
          with pkgs;
          mkShell rec {
            buildInputs = [
              (rust-bin.stable.latest.minimal.override {
                extensions = [
                  "clippy"
                  "rust-analyzer"
                  "rust-docs"
                  "rust-src"
                ];
              })
              (rust-bin.selectLatestNightlyWith (toolchain: toolchain.rustfmt))

              alsa-lib
              alsa-utils
              pulseaudio
              pipewire

              shaderc
              spirv-tools
              vulkan-loader
              vulkan-tools
              vulkan-tools-lunarg
              vulkan-validation-layers

              libGL

              libxkbcommon
              wayland
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr

              bashInteractive
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
            PKG_CONFIG_PATH = "${alsa-lib.dev}/lib/pkgconfig";
            SHADERC_LIB_DIR = lib.makeLibraryPath [ shaderc ];
            VK_LAYER_PATH = "${vulkan-validation-layers}/share/vulkan/explicit_layer.d";

            shellHook = '' export SHELL="${pkgs.bashInteractive}/bin/bash"; '';
          };
      }
    );
}

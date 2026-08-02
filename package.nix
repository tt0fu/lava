{
  lib,
  rustPlatform,
  pkg-config,
  alsa-lib,
  shaderc,
  jack2,
  vulkan-loader,
  libxkbcommon,
  wayland,

  portable ? false,
  ...
}:
rustPlatform.buildRustPackage rec {
  pname = "lava";
  version = (fromTOML (builtins.readFile ./Cargo.toml)).package.version;
  doCheck = false;
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "vulkano-0.35.0" = "sha256-UMfCh67b/Yb4w7EcN+G2z+BCkOR4ecuElgllBdN4nxY=";
      "concurrent-slotmap-0.1.0-alpha.2" = "sha256-Sle4tcFvWLLKmghpzG6Ds/yU57VrNddzDIerVJ8eHd0=";
    };
  };
  src = ./.;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    vulkan-loader
    libxkbcommon
    wayland
    alsa-lib
    jack2
  ];

  env = {
    PKG_CONFIG_PATH = "${alsa-lib.dev}/lib/pkgconfig:${jack2.dev}/lib/pkgconfig";
    SHADERC_LIB_DIR = lib.makeLibraryPath [ shaderc ];
  };

  postFixup =
    if portable then
      ''
        patchelf \
          --set-interpreter /lib64/ld-linux-x86-64.so.2 \
          $out/bin/lava
      ''
    else
      ''
        patchelf \
          --set-rpath ${lib.makeLibraryPath buildInputs} \
          $out/bin/lava
      '';
}

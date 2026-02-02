{
  rustPlatform,
  lib,
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
  version = "0.0.2";
  doCheck = false;
  cargoLock = {
    lockFile = ./Cargo.lock;
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

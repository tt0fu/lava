{
  rustPlatform,
  lib,
  pkg-config,
  makeWrapper,
  alsa-lib,
  shaderc,
  jack2,
  vulkan-loader,
  libxkbcommon,
  wayland,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "lava";
  version = "0.0.1";
  doCheck = false;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  src = ./.;
  nativeBuildInputs = [
    pkg-config
    makeWrapper

    alsa-lib
    shaderc
    jack2
  ];

  env = {
    PKG_CONFIG_PATH = "${alsa-lib.dev}/lib/pkgconfig:${jack2.dev}/lib/pkgconfig";
    SHADERC_LIB_DIR = lib.makeLibraryPath [ shaderc ];
  };
  
  postFixup = ''
    wrapProgram $out/bin/lava \
      --set LD_LIBRARY_PATH ${
        lib.makeLibraryPath [
          vulkan-loader
          libxkbcommon
          wayland
        ]
      }
  '';
}

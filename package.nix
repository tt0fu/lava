{
  stdenv,
  rustPlatform,
  lib,
  pkg-config,
  alsa-lib,
  cmake,
  git,
  shaderc,
  jack2,
  python3,
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
  };
  src = ./.;

  cargoBuildTarget = stdenv.targetPlatform.rust.rustcTarget;

  nativeBuildInputs = [
    pkg-config
  ]
  ++ lib.optionals stdenv.targetPlatform.isWindows [
    cmake
    git
    python3
    stdenv.cc
  ];

  buildInputs = lib.optionals stdenv.isLinux [
    vulkan-loader
    libxkbcommon
    wayland
    alsa-lib
    jack2
  ];

  env =
    if stdenv.isLinux then
      {
        PKG_CONFIG_PATH = "${alsa-lib.dev}/lib/pkgconfig:${jack2.dev}/lib/pkgconfig";
        SHADERC_LIB_DIR = lib.makeLibraryPath [ shaderc ];
      }
    else
      {
        CARGO_BUILD_TARGET = stdenv.targetPlatform.rust.rustcTarget;

        CMAKE_SYSTEM_NAME = "Windows";
        CMAKE_SYSTEM_PROCESSOR = "x86_64";
        CMAKE_POLICY_VERSION_MINIMUM = "3.5";
        CXXFLAGS = "-std=c++17";

        CC = "${stdenv.cc.targetPrefix}cc";
        CXX = "${stdenv.cc.targetPrefix}c++";
        AR = "${stdenv.cc.targetPrefix}ar";
      };

  postFixup =
    if stdenv.isLinux then
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
        ''
    else
      "";
}

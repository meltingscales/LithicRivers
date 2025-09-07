{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.clippy
    pkgs.pkg-config
    pkgs.alsa-lib
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXrandr
    pkgs.xorg.libXrender
    pkgs.xorg.libXi
    pkgs.wayland
    pkgs.openssl
    pkgs.libGL
    pkgs.udev
    pkgs.vulkan-loader
    pkgs.mesa
    pkgs.mold
    pkgs.binutils
    pkgs.udev
    pkgs.tracy
    pkgs.cargo-flamegraph
    pkgs.alsa-lib
    pkgs.alsa-oss
    pkgs.steamcmd
    pkgs.graphviz
  ];
  shellHook = ''
    export PKG_CONFIG_PATH=${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH
    export LD_LIBRARY_PATH=${pkgs.xorg.libX11}/lib:${pkgs.xorg.libXcursor}/lib:${pkgs.xorg.libXrandr}/lib:${pkgs.xorg.libXrender}/lib:${pkgs.xorg.libXi}/lib:${pkgs.vulkan-loader}/lib:${pkgs.mesa}/lib:${pkgs.udev}/lib:${pkgs.alsa-lib}/lib:$LD_LIBRARY_PATH
  '';
}

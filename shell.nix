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
    pkgs.wayland
    pkgs.openssl
    pkgs.libGL
    pkgs.udev
  ];
  shellHook = ''
    export PKG_CONFIG_PATH=${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH
  '';
}

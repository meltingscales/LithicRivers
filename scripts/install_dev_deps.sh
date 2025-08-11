#!/usr/bin/env bash
set -euo pipefail
if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update && sudo apt-get install -y \
    pkg-config libasound2-dev libudev-dev \
    libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxkbcommon-dev \
    libwayland-dev wayland-protocols libgl1-mesa-dev
elif command -v dnf >/dev/null 2>&1; then
  sudo dnf install -y \
    pkgconf-pkg-config alsa-lib-devel libudev-devel \
    libX11-devel libXi-devel libXrandr-devel libXcursor-devel libxkbcommon-devel \
    wayland-devel wayland-protocols-devel mesa-libGL-devel
elif command -v pacman >/dev/null 2>&1; then
  sudo pacman -Sy --needed --noconfirm \
    pkgconf alsa-lib libudev0-shim \
    libx11 libxi libxrandr libxcursor libxkbcommon \
    wayland wayland-protocols mesa
elif command -v apk >/dev/null 2>&1; then
  sudo apk add \
    pkgconfig alsa-lib-dev eudev-dev \
    libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxkbcommon-dev \
    wayland-dev wayland-protocols mesa-dev
else
  echo "Unsupported package manager. Please install pkg-config and the X/Wayland/ALSA dev packages manually." >&2
  exit 0
fi

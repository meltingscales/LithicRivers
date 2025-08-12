#!/usr/bin/env bash
set -euo pipefail

SUDO=""
if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  SUDO="sudo"
fi

if command -v apt-get >/dev/null 2>&1; then
  # Smarter APT path: only install missing packages
  # Base dev deps for X/Wayland/ALSA + GL
  APT_PACKAGES=(
    pkg-config
    libasound2-dev
    libudev-dev
    libx11-dev
    libxi-dev
    libxrandr-dev
    libxcursor-dev
    libxkbcommon-dev
    libwayland-dev
    wayland-protocols
    libgl1-mesa-dev
  )

  MISSING=()
  for pkg in "${APT_PACKAGES[@]}"; do
    if ! dpkg-query -W -f='${Status}\n' "$pkg" 2>/dev/null | grep -q "install ok installed"; then
      MISSING+=("$pkg")
    fi
  done

  if ((${#MISSING[@]})); then
    echo "Installing missing APT packages: ${MISSING[*]}"
    DEBIAN_FRONTEND=noninteractive $SUDO apt-get update -y
    DEBIAN_FRONTEND=noninteractive $SUDO apt-get install -y --no-install-recommends "${MISSING[@]}"
  else
    echo "All APT packages already installed."
  fi
elif command -v dnf >/dev/null 2>&1; then
  # Smarter DNF path: only install missing packages
  DNF_PACKAGES=(
    pkgconf-pkg-config
    alsa-lib-devel
    libudev-devel
    libX11-devel
    libXi-devel
    libXrandr-devel
    libXcursor-devel
    libxkbcommon-devel
    wayland-devel
    wayland-protocols-devel
    mesa-libGL-devel
  )

  MISSING=()
  for pkg in "${DNF_PACKAGES[@]}"; do
    if ! rpm -q "$pkg" >/dev/null 2>&1; then
      MISSING+=("$pkg")
    fi
  done

  if ((${#MISSING[@]})); then
    echo "Installing missing DNF packages: ${MISSING[*]}"
    $SUDO dnf install -y --setopt=install_weak_deps=False "${MISSING[@]}"
  else
    echo "All DNF packages already installed."
  fi
elif command -v pacman >/dev/null 2>&1; then
  # Smarter pacman path: only install missing packages, sync DB if needed
  PACMAN_PACKAGES=(
    pkgconf
    alsa-lib
    libudev0-shim
    libx11
    libxi
    libxrandr
    libxcursor
    libxkbcommon
    wayland
    wayland-protocols
    mesa
  )

  # pacman -T lists targets that are not installed
  # shellcheck disable=SC2207
  MISSING=($(pacman -T "${PACMAN_PACKAGES[@]}" 2>/dev/null || true))
  if ((${#MISSING[@]})); then
    echo "Installing missing pacman packages: ${MISSING[*]}"
    $SUDO pacman -Sy --needed --noconfirm "${MISSING[@]}"
  else
    echo "All pacman packages already installed."
  fi
elif command -v apk >/dev/null 2>&1; then
  # Smarter Alpine apk path: only install missing packages
  APK_PACKAGES=(
    pkgconfig
    alsa-lib-dev
    eudev-dev
    libx11-dev
    libxi-dev
    libxrandr-dev
    libxcursor-dev
    libxkbcommon-dev
    wayland-dev
    wayland-protocols
    mesa-dev
  )

  MISSING=()
  for pkg in "${APK_PACKAGES[@]}"; do
    if ! apk info -e "$pkg" >/dev/null 2>&1; then
      MISSING+=("$pkg")
    fi
  done

  if ((${#MISSING[@]})); then
    echo "Installing missing apk packages: ${MISSING[*]}"
    $SUDO apk add "${MISSING[@]}"
  else
    echo "All apk packages already installed."
  fi
else
  echo "Unsupported package manager. Please install pkg-config and the X/Wayland/ALSA dev packages manually." >&2
  exit 0
fi

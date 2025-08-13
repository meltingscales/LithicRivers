#!/usr/bin/env bash
set -euo pipefail

SUDO=""
if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  SUDO="sudo"
fi

# Best-effort: grant perf capabilities to allow non-root profiling
if command -v perf >/dev/null 2>&1 && command -v setcap >/dev/null 2>&1; then
  echo "Setting capabilities on $(command -v perf) (best-effort)"
  $SUDO setcap cap_perfmon,cap_sys_ptrace,cap_sys_admin+ep "$(command -v perf)" || true
else
  echo "Note: perf or setcap not available; you may need to run profiling with sudo or install libcap tools."
fi

cat <<'TIP'
Tips for enabling perf without sudo (optional):
  sudo sysctl kernel.perf_event_paranoid=1
  sudo sysctl kernel.kptr_restrict=0
Persist across reboots:
  echo "kernel.perf_event_paranoid=1" | sudo tee /etc/sysctl.d/99-perf.conf
  echo "kernel.kptr_restrict=0" | sudo tee -a /etc/sysctl.d/99-perf.conf
  sudo sysctl --system
TIP

if command -v apt-get >/dev/null 2>&1; then
  # Smarter APT path: only install missing packages
  # Base dev deps for X/Wayland/ALSA + GL
  APT_PACKAGES=(
    build-essential
    binutils
    mold
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

  # Perf tooling for cargo-flamegraph
  # Try kernel-specific tools first, then fall back to generic meta-packages
  KREL=$(uname -r)
  PERF_PKGS=("linux-tools-${KREL}" "linux-cloud-tools-${KREL}" linux-tools-generic linux-cloud-tools-generic libcap2-bin)
  PERF_MISSING=()
  for pkg in "${PERF_PKGS[@]}"; do
    if $SUDO apt-cache show "$pkg" >/dev/null 2>&1; then
      if ! dpkg-query -W -f='${Status}\n' "$pkg" 2>/dev/null | grep -q "install ok installed"; then
        PERF_MISSING+=("$pkg")
      fi
    fi
  done
  if ((${#PERF_MISSING[@]})); then
    echo "Installing perf support packages: ${PERF_MISSING[*]}"
    DEBIAN_FRONTEND=noninteractive $SUDO apt-get install -y --no-install-recommends "${PERF_MISSING[@]}" || true
  fi
elif command -v dnf >/dev/null 2>&1; then
  # Smarter DNF path: only install missing packages
  DNF_PACKAGES=(
    gcc
    gcc-c++
    make
    glibc-devel
    binutils
    mold
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

  # Perf and capabilities for Fedora/RHEL
  DNF_EXTRA=(perf libcap)
  DNF_NEED=()
  for pkg in "${DNF_EXTRA[@]}"; do
    if ! rpm -q "$pkg" >/dev/null 2>&1; then
      DNF_NEED+=("$pkg")
    fi
  done
  if ((${#DNF_NEED[@]})); then
    echo "Installing perf/capabilities packages: ${DNF_NEED[*]}"
    $SUDO dnf install -y --setopt=install_weak_deps=False "${DNF_NEED[@]}" || true
  fi
elif command -v pacman >/dev/null 2>&1; then
  # Smarter pacman path: only install missing packages, sync DB if needed
  PACMAN_PACKAGES=(
    gcc
    make
    binutils
    mold
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

  # Perf and capabilities for Arch
  PACMAN_EXTRA=(perf libcap)
  # shellcheck disable=SC2207
  NEED=($(pacman -T "${PACMAN_EXTRA[@]}" 2>/dev/null || true))
  if ((${#NEED[@]})); then
    echo "Installing perf/capabilities packages: ${NEED[*]}"
    $SUDO pacman -Sy --needed --noconfirm "${NEED[@]}" || true
  fi
elif command -v apk >/dev/null 2>&1; then
  # Smarter Alpine apk path: only install missing packages
  APK_PACKAGES=(
    build-base
    mold
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

  # Perf and capabilities for Alpine (package names may vary by repo)
  APK_EXTRA=()
  if apk search -e perf >/dev/null 2>&1; then APK_EXTRA+=(perf); fi
  if apk search -e libcap >/dev/null 2>&1; then APK_EXTRA+=(libcap); fi
  if ((${#APK_EXTRA[@]})); then
    for pkg in "${APK_EXTRA[@]}"; do
      if ! apk info -e "$pkg" >/dev/null 2>&1; then
        echo "Installing $pkg"
        $SUDO apk add "$pkg" || true
      fi
    done
  fi
else
  echo "Unsupported package manager. Please install pkg-config and the X/Wayland/ALSA dev packages manually." >&2
  exit 0
fi

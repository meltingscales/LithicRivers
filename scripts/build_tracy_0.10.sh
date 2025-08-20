#!/usr/bin/env bash
set -euo pipefail

# You can download the tracy 0.10 tree at:
# https://github.com/wolfpld/tracy/archive/refs/tags/v0.10.zip

# Tracy UI build helper for the 0.10 tree (Makefile-based UI)
# Builds the profiler UI from profiler/build/unix and optionally installs the binary.
#
print_usage() {
    cat <<EOF
Usage: $0 [options]

Options:
    --help           Show this help message
    --legacy         Build the X11 backend (default: Wayland)
    --debug          Build a debug version (default: release)
    --jobs <N>       Number of jobs to use for parallel build (default: auto)
    --install <mode> Install the binary (system|user|path)
    --install-path <path> Install the binary to a specific path
    --name <name>    Custom installed filename (default: Tracy)
    --no-lto         Disable LTO
    --gtk-fileselector Use GTK file chooser instead of xdg-portal
    --no-fileselector Disable file chooser entirely

Notes:
    - Wayland is default on Linux in this tree. Use --legacy to build the X11 backend.
    - Dependencies (pkg-config, g++, make, freetype2, capstone, wayland/x11 headers, etc.) must be installed.
EOF
}

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
UI_DIR="${ROOT_DIR}/profiler/build/unix"
ARTIFACT_WAYLAND="${UI_DIR}/Tracy-release"
ARTIFACT_DEBUG="${UI_DIR}/Tracy-debug"

# Defaults
LEGACY=0
BUILD_TYPE=release
JOBS="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"
INSTALL_MODE=""   # empty|system|user|path
INSTALL_PATH=""   # used when INSTALL_MODE=path
INSTALL_NAME="Tracy"
TRACY_NO_LTO=0
TRACY_GTK_FILESELECTOR=0
TRACY_NO_FILESELECTOR=0


while [[ ${1:-} =~ ^- ]]; do
  case "$1" in
    -h|--help) print_usage; exit 0 ;;
    --legacy) LEGACY=1 ;;
    --debug) BUILD_TYPE=debug ;;
    --release) BUILD_TYPE=release ;;
    -j|--jobs) shift; JOBS="${1:?--jobs requires a value}" ;;
    --install) shift; INSTALL_MODE="${1:?--install requires system|user|path}" ;;
    --install-path) shift; INSTALL_MODE=path; INSTALL_PATH="${1:?--install-path requires a path}" ;;
    --name) shift; INSTALL_NAME="${1:?--name requires a value}" ;;
    --no-lto) TRACY_NO_LTO=1 ;;
    --gtk-fileselector) TRACY_GTK_FILESELECTOR=1 ;;
    --no-fileselector) TRACY_NO_FILESELECTOR=1 ;;
    --) shift; break ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
  shift || true
done

# Basic sanity checks
command -v make >/dev/null || { echo "make not found" >&2; exit 1; }
command -v g++ >/dev/null || { echo "g++ not found" >&2; exit 1; }
command -v pkg-config >/dev/null || { echo "pkg-config not found" >&2; exit 1; }

if [[ ! -d "$UI_DIR" ]]; then
  echo "UI directory not found: $UI_DIR" >&2
  exit 1
fi

# Compose make arguments
MAKE_ARGS=( -C "$UI_DIR" -j "$JOBS" )

# Map build type to Makefile target
case "$BUILD_TYPE" in
  release) MAKE_TARGET=release ;;
  debug)   MAKE_TARGET=debug ;;
  *) echo "Invalid build type: $BUILD_TYPE" >&2; exit 2 ;;
esac

# Environment toggles consumed by the makefiles
ENV_VARS=()
[[ "$LEGACY" -eq 1 ]] && ENV_VARS+=( LEGACY=1 )
[[ "$TRACY_NO_LTO" -eq 1 ]] && ENV_VARS+=( TRACY_NO_LTO=1 )
[[ "$TRACY_GTK_FILESELECTOR" -eq 1 ]] && ENV_VARS+=( TRACY_GTK_FILESELECTOR=1 )
[[ "$TRACY_NO_FILESELECTOR" -eq 1 ]] && ENV_VARS+=( TRACY_NO_FILESELECTOR=1 )

# Build
echo "==> Building UI ($BUILD_TYPE) LEGACY=$LEGACY LTO=$((TRACY_NO_LTO==0)) GTK_FILESELECTOR=$TRACY_GTK_FILESELECTOR NO_FILESELECTOR=$TRACY_NO_FILESELECTOR"
make "${ENV_VARS[@]}" "${MAKE_ARGS[@]}" "$MAKE_TARGET"

# Determine artifact path
if [[ "$BUILD_TYPE" == "release" ]]; then
  ARTIFACT="$ARTIFACT_WAYLAND"
else
  ARTIFACT="$ARTIFACT_DEBUG"
fi

if [[ ! -f "$ARTIFACT" ]]; then
  # Fallback: look for Tracy-$BUILD_TYPE
  ALT_ARTIFACT="${UI_DIR}/Tracy-${BUILD_TYPE}"
  if [[ -f "$ALT_ARTIFACT" ]]; then
    ARTIFACT="$ALT_ARTIFACT"
  fi
fi

if [[ ! -f "$ARTIFACT" ]]; then
  echo "Build completed but artifact not found in $UI_DIR" >&2
  exit 1
fi

echo "==> Built: $ARTIFACT"

# Optional installation
case "$INSTALL_MODE" in
  "") ;; # no install
  system)
    echo "==> Installing system-wide to /usr/local/bin/$INSTALL_NAME (may require sudo)"
    install -m 0755 -D "$ARTIFACT" "/usr/local/bin/$INSTALL_NAME" 2>/dev/null || {
      echo "Attempting with sudo..."
      sudo install -m 0755 -D "$ARTIFACT" "/usr/local/bin/$INSTALL_NAME"
    }
    echo "Installed at /usr/local/bin/$INSTALL_NAME"
    ;;
  user)
    : "${XDG_BIN_HOME:=${HOME}/.local/bin}"
    mkdir -p "$XDG_BIN_HOME"
    install -m 0755 -D "$ARTIFACT" "$XDG_BIN_HOME/$INSTALL_NAME"
    echo "Installed at $XDG_BIN_HOME/$INSTALL_NAME"
    if ! command -v "$INSTALL_NAME" >/dev/null 2>&1; then
      echo "Note: Ensure $XDG_BIN_HOME is on your PATH" >&2
    fi
    ;;
  path)
    if [[ -z "$INSTALL_PATH" ]]; then
      echo "--install path requires --install-path <dir|file>" >&2
      exit 2
    fi
    # If INSTALL_PATH is a directory, install as $INSTALL_NAME in that dir; else install to exact file path
    if [[ -d "$INSTALL_PATH" ]]; then
      DEST="$INSTALL_PATH/$INSTALL_NAME"
    else
      mkdir -p "$(dirname "$INSTALL_PATH")"
      DEST="$INSTALL_PATH"
    fi
    install -m 0755 -D "$ARTIFACT" "$DEST"
    echo "Installed at $DEST"
    ;;
  *)
    echo "Invalid --install mode: $INSTALL_MODE (expected system|user|path)" >&2
    exit 2
    ;;
esac

echo "==> Done. You can run it with: $INSTALL_NAME (if installed) or $ARTIFACT"

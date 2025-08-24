#!/usr/bin/env bash

# check if STEAMCMD_PASSWORD is set
if [ -z "$STEAMCMD_PASSWORD" ]; then
  echo "Error: STEAMCMD_PASSWORD is not set. Please set it in the environment."
  echo "Example: export STEAMCMD_PASSWORD=your_password && ./scripts/steam_release_manual.sh"
  exit 1
fi

make build stage-artifacts

APPVDF="$(realpath app_build_linux.vdf)"

steamcmd +login "lithicriversbuild" "$STEAMCMD_PASSWORD" +run_app_build_http "$APPVDF" +quit
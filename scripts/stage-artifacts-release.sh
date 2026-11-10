#!/usr/bin/env bash
# Stage artifacts release

cp -f target/release/lithicrivers-client artifacts/
cp -f target/release/demo_* artifacts/
cp -f target/release/utility_* artifacts/
cp -f scripts/launcher/lithicrivers-launcher.sh artifacts/

# The Steam API library the release binary loads (libsteam_api.so on Linux,
# libsteam_api.dylib on macOS), from where the steamworks-sys build left it
steam_lib=$(find target/release/build -path '*steamworks-sys*' \( -name 'libsteam_api.so' -o -name 'libsteam_api.dylib' \) | head -n 1)
if [ -z "$steam_lib" ]; then
    echo "Steam API library not found; was the release built with --features steam?" >&2
    exit 1
fi
cp -f "$steam_lib" artifacts/

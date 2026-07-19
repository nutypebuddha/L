#!/usr/bin/env bash
# install-lai.sh — Install L.ai assistant on Termux
# Run this from Termux (not proot)
set -euo pipefail

echo "=== L.ai Assistant Installer ==="

# Check if we're in Termux
if [ -z "${PREFIX:-}" ]; then
    echo "Error: run this from Termux, not proot"
    exit 1
fi

# Install termux-api if missing
if ! command -v termux-battery-status &>/dev/null; then
    echo "Installing termux-api..."
    pkg install -y termux-api
fi

# Test termux-api
echo "Testing termux-api..."
termux-battery-status > /dev/null 2>&1 && echo "  ✓ termux-api working" || echo "  ⚠ termux-api not responding (open Termux:API app)"

# Find the binary
LAI_BIN=""
for candidate in \
    "$PREFIX/bin/lai" \
    "$HOME/lai" \
    "/sdcard/Download/lai" \
    "$HOME/Laverna/target/release/lai" \
    "$HOME/lai/target/release/lai"; do
    if [ -f "$candidate" ]; then
        LAI_BIN="$candidate"
        break
    fi
done

if [ -z "$LAI_BIN" ]; then
    echo "Error: lai binary not found"
    echo "Copy it to \$PREFIX/bin/ or \$HOME/"
    exit 1
fi

# Install to PATH
if [ "$LAI_BIN" != "$PREFIX/bin/lai" ]; then
    echo "Installing to $PREFIX/bin/lai..."
    cp "$LAI_BIN" "$PREFIX/bin/lai"
    chmod +x "$PREFIX/bin/lai"
fi

echo ""
echo "=== Installed ==="
lai --version
echo ""
echo "Quick test:"
lai assistant --text "set a 2 minute timer"
echo ""
echo "Usage:"
echo "  lai assistant --text \"set a 5 minute timer\""
echo "  lai assistant --text \"text John hello\""
echo "  lai assistant --text \"what's my battery level\""
echo "  lai assistant --text \"take a photo\""
echo "  lai assistant --text \"where am I\""
echo "  lai assistant --voice          (needs whisper + espeak)"

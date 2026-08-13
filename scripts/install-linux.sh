#!/usr/bin/env bash
# install-linux.sh — Install L.ai on any Linux (x86_64 / aarch64)
#
# Downloads the prebuilt static binary from GitHub releases and drops it on PATH.
# No Rust toolchain required. For Termux use scripts/install-termux.sh instead.
#
# Usage:
#   ./install-linux.sh                 # install to /usr/local/bin (or ~/.local/bin)
#   PREFIX=/opt/bin ./install-linux.sh # install to a custom dir
#   LAI_VERSION=v0.4.2 ./install-linux.sh   # pin a release
set -euo pipefail

REPO="nutypebuddha/L"
ASSET_BASE="lai"

# 1. Detect architecture
arch="$(uname -m)"
case "$arch" in
    x86_64|amd64)  bin="lai-x86_64" ;;
    aarch64|arm64) bin="lai-aarch64" ;;
    *) echo "Error: unsupported architecture '$arch' (need x86_64 or aarch64)" >&2; exit 1 ;;
esac

# 2. Resolve destination
if [ -n "${PREFIX:-}" ]; then
    DEST_DIR="$PREFIX"
elif [ -w /usr/local/bin ]; then
    DEST_DIR="/usr/local/bin"
else
    DEST_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
    mkdir -p "$DEST_DIR"
    case ":$PATH:" in
        *":$DEST_DIR:"*) ;;
        *) echo "Note: add $DEST_DIR to your PATH (e.g. export PATH=\"$DEST_DIR:\$PATH\")" >&2 ;;
    esac
fi
DEST="$DEST_DIR/lai"

# 3. Resolve version (latest release tag unless pinned)
if [ -n "${LAI_VERSION:-}" ]; then
    tag="$LAI_VERSION"
else
    echo "Resolving latest release..." >&2
    tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
            | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
    if [ -z "$tag" ]; then
        echo "Error: could not determine latest release (network?). Set LAI_VERSION=vX.Y.Z" >&2
        exit 1
    fi
fi
url="https://github.com/$REPO/releases/download/$tag/$bin"

# 4. Download
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
echo "Downloading $url" >&2
if ! curl -fsSL "$url" -o "$tmp"; then
    echo "Error: download failed. The static binary for $tag/$bin may not exist yet." >&2
    exit 1
fi

# 5. Verify checksum if the release published one (brand: verify, don't trust)
sum_url="https://github.com/$REPO/releases/download/$tag/$bin.sha256"
if sum="$(curl -fsSL "$sum_url" 2>/dev/null)"; then
    echo "$sum" > "$tmp.sha256"
    if sha256sum -c "$tmp.sha256" --status 2>/dev/null; then
        echo "Checksum verified." >&2
    else
        echo "Error: checksum mismatch for downloaded $bin" >&2
        exit 1
    fi
else
    echo "Warning: no $bin.sha256 published for $tag — skipping verification" >&2
fi

# 6. Install
install -m 0755 "$tmp" "$DEST"
echo "Installed: $DEST" >&2

# 6. Smoke test
if "$DEST" ping >/dev/null 2>&1; then
    echo "OK: $("$DEST" --version 2>/dev/null || echo lai) installed and responding."
    echo "Try: lai info   |   lai solve --query \"2 + 3 = 5\"   |   lai --help"
else
    echo "Installed, but 'lai ping' failed — the binary may need libraries absent on this host." >&2
    exit 1
fi

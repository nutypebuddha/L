#!/data/data/com.termux/files/usr/bin/bash
# Install L.ai from proot build to Termux
set -e

SRC="/tmp/lai"
DEST="$PREFIX/bin/lai"

if [ ! -f "$SRC" ]; then
    echo "Error: $SRC not found"
    echo "Build first from proot: cargo build --release -p laverna --features assistant"
    exit 1
fi

cp "$SRC" "$DEST"
chmod 755 "$DEST"
echo "Installed: $DEST"
echo "Run: lai assistant --text \"help\""

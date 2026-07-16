#!/bin/bash
# L.ai · Proof (Laverna) WASM build script.
# Mirrors cid-wasm: builds the cdylib for wasm32-unknown-unknown and emits www/.

set -e

echo "🔧 Building L.ai · Proof (laverna-wasm)..."

. "$HOME/.cargo/env" 2>/dev/null || true

cd "$(dirname "$0")"

echo "📦 Cleaning previous builds..."
cargo clean

echo "🏗️  Compiling for wasm32-unknown-unknown..."
cargo build --target wasm32-unknown-unknown --release

WASM_FILE="target/wasm32-unknown-unknown/release/laverna_wasm.wasm"

if command -v wasm-bindgen &> /dev/null; then
    echo "📄 Generating JavaScript bindings with wasm-bindgen..."
    wasm-bindgen "$WASM_FILE" --out-dir www --target web
    echo "✅ Generated www/laverna_wasm.js and www/laverna_wasm_bg.wasm"
else
    echo "⚠️  wasm-bindgen not found. Using raw WASM."
    echo "   cargo install wasm-bindgen-cli --locked"
    mkdir -p www
    cp "$WASM_FILE" www/ 2>/dev/null || true
fi

cp index.html www/ 2>/dev/null || true
cp loader.js www/ 2>/dev/null || true

if [ -f "$WASM_FILE" ]; then
    SIZE=$(ls -lh "$WASM_FILE" | awk '{print $5}')
    echo ""
    echo "✅ Build complete! WASM size: $SIZE"
    echo "🌐 Test: cd www && python3 -m http.server 8080"
fi

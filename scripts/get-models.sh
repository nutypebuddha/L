#!/usr/bin/env bash
# get-models.sh — Download speech models for L.ai Assistant
# Run from the repo root or from bin/models/
set -euo pipefail

MODELS_DIR="${1:-bin/models}"
mkdir -p "$MODELS_DIR"

echo "=== L.ai Assistant — Model Downloader ==="
echo "Downloading to: $MODELS_DIR"
echo ""

# ── Whisper STT model (tiny.en — ~75MB, good for aarch64) ────────
WHISPER_MODEL="$MODELS_DIR/whisper-tiny.en.bin"
if [ ! -f "$WHISPER_MODEL" ]; then
    echo "Downloading whisper-tiny.en (STT model, ~75MB)..."
    curl -L -o "$WHISPER_MODEL" \
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
    echo "  ✓ whisper-tiny.en saved to $WHISPER_MODEL"
else
    echo "  ✓ whisper-tiny.en already exists"
fi

# ── Piper TTS model (en_US-lessac-medium — ~50MB) ────────────────
TTS_MODEL="$MODELS_DIR/piper-en_US-lessac-medium.onnx"
TTS_CONFIG="$MODELS_DIR/piper-en_US-lessac-medium.onnx.json"
if [ ! -f "$TTS_MODEL" ]; then
    echo "Downloading piper-en_US-lessac-medium (TTS model, ~50MB)..."
    curl -L -o "$TTS_MODEL" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx"
    curl -L -o "$TTS_CONFIG" \
        "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json"
    echo "  ✓ piper-en_US-lessac-medium saved"
else
    echo "  ✓ piper-en_US-lessac-medium already exists"
fi

# ── espeak-ng data (required by piper-rs for phonemization) ──────
ESPEAK_DIR="$MODELS_DIR/espeak-ng-data"
if [ ! -d "$ESPEAK_DIR" ]; then
    echo "Downloading espeak-ng data (TTS phonemizer)..."
    mkdir -p "$ESPEAK_DIR"
    curl -L -o /tmp/espeak-ng-data.tar.gz \
        "https://github.com/thewh1teagle/piper-rs/releases/download/espeak-ng-files/espeak-ng-data.tar.gz"
    tar -xzf /tmp/espeak-ng-data.tar.gz -C "$ESPEAK_DIR"
    rm -f /tmp/espeak-ng-data.tar.gz
    echo "  ✓ espeak-ng-data saved"
else
    echo "  ✓ espeak-ng-data already exists"
fi

echo ""
echo "=== Done ==="
echo "Models are in: $MODELS_DIR"
echo ""
echo "To test:"
echo "  cargo build --release --features assistant -p laverna"
echo "  ./target/release/lai assistant --text \"what's my battery level\""

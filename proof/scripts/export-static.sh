#!/usr/bin/env bash
# Build a small, portable, STATIC release binary and export it to a hub dir.
#
# Unlike export.sh (which needs an x86_64 musl cross-gcc + sysroot and targets
# /sdcard), this script is toolchain-light: it links with Rust's bundled
# `rust-lld`, so the default C-free feature set builds for either arch with no
# external cross toolchain. The result is then UPX-compressed (self-decompresses
# at launch) to cut ~75% of on-disk size.
#
# Sizes observed (default features, opt-level=z + fat LTO + strip, then UPX):
#   x86_64-unknown-linux-musl : 9.3 MB -> 2.3 MB
#   aarch64-unknown-linux-musl: 7.8 MB -> 2.3 MB
#
# Requirements:
#   * rustup target add <TARGET>              (musl std for the chosen arch)
#   * upx on PATH, or set UPX=/path/to/upx    (skips compression if absent)
#   * PROTOC / protoc on PATH (build-dep)
#
# Env:
#   ARCH               x86_64 (default) | aarch64
#   FEATURES           cargo features (default: the crate's default set)
#   HUB                export dir (default: $HOME/downloads)
#   UPX_BIN            path to the upx binary (default: `command -v upx`)
#   NO_UPX=1           skip compression, export the raw static binary
#   CARGO_BUILD_JOBS   parallel jobs (not hardcoded; set per-invocation)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE_ROOT="$(cd "$REPO_ROOT/.." && pwd)"
cd "$WORKSPACE_ROOT"

ARCH="${ARCH:-x86_64}"
case "$ARCH" in
    x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
    aarch64) TARGET="aarch64-unknown-linux-musl" ;;
    *) echo "error: ARCH must be x86_64 or aarch64 (got '$ARCH')" >&2; exit 1 ;;
esac

HUB="${HUB:-${HOME:-/root}/downloads}"
DEST="$HUB/lai-${ARCH}-static"
# NB: upx reads its own options from the `UPX` env var, so we never name ours
# that. Also scrub any inherited `UPX` so it can't corrupt the upx invocation.
UPX_BIN="${UPX_BIN:-$(command -v upx || true)}"
unset UPX

# Link with Rust's self-contained LLD so no external cross-gcc is needed for the
# default (C-free) feature set. Per-target flag: host builds stay untouched.
RUSTFLAGS_VAR="CARGO_TARGET_$(echo "$TARGET" | tr 'a-z-' 'A-Z_')_RUSTFLAGS"
export "$RUSTFLAGS_VAR=-C linker=rust-lld -C linker-flavor=ld.lld"

echo "==> disk before build"
df -h / | tail -1

echo "==> building static $TARGET${FEATURES:+ (features: $FEATURES)}"
if [ -n "${FEATURES:-}" ]; then
    cargo build --release --target "$TARGET" -p laverna --features "$FEATURES"
else
    cargo build --release --target "$TARGET" -p laverna
fi

BIN="$WORKSPACE_ROOT/target/$TARGET/release/lai"
[ -f "$BIN" ] || { echo "error: build produced no binary at $BIN" >&2; exit 1; }

echo "==> verifying static linkage"
if od -c "$BIN" 2>/dev/null | grep -aq 'ld-linux'; then
    echo "warn: binary references a dynamic linker -> NOT fully static"
else
    echo "ok: no dynamic-linker reference -> STATIC"
fi

mkdir -p "$HUB"
cp "$BIN" "$DEST"
chmod +x "$DEST"
RAW_SIZE="$(du -h "$DEST" | cut -f1)"

if [ "${NO_UPX:-0}" = 1 ]; then
    echo "==> NO_UPX=1 set; skipping compression"
elif [ -n "$UPX_BIN" ] && [ -x "$UPX_BIN" ]; then
    echo "==> compressing with UPX ($UPX_BIN)"
    "$UPX_BIN" --best --lzma "$DEST"
else
    echo "warn: upx not found (set UPX=/path/to/upx or NO_UPX=1); shipping raw binary"
fi

echo "==> done: $DEST (raw $RAW_SIZE -> $(du -h "$DEST" | cut -f1))"
ls -lh "$DEST"

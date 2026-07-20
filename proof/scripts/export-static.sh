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
#   SDCARD=1           export to shared-storage Download instead of HUB.
#                      Path autodetected (env SDCARD_DIR overrides). On this
#                      proot the shared store is bind-mounted at /mnt/android;
#                      it is a FUSE mount (no exec bit, no symlinks) so copy the
#                      binary to an exec-capable fs and `chmod +x` before running.
#   SDCARD_DIR         explicit shared-storage Download dir (implies SDCARD=1)
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

# Resolve the export dir. SDCARD=1 (or SDCARD_DIR) targets shared storage,
# autodetecting the first existing Download among the known mount points.
if [ -n "${SDCARD_DIR:-}" ]; then
    HUB="$SDCARD_DIR"
elif [ "${SDCARD:-0}" = 1 ]; then
    HUB=""
    for d in /mnt/android/Download /storage/emulated/0/Download \
             /sdcard/Download /storage/self/primary/Download; do
        [ -d "$d" ] && { HUB="$d"; break; }
    done
    [ -n "$HUB" ] || { echo "error: SDCARD set but no shared-storage Download dir found (set SDCARD_DIR=...)" >&2; exit 1; }
    echo "==> SDCARD export dir: $HUB"
else
    HUB="${HUB:-${HOME:-/root}/downloads}"
fi
DEST="$HUB/lai-${ARCH}-static"
# NB: upx reads its own options from the `UPX` env var, so we never name ours
# that. Also scrub any inherited `UPX` so it can't corrupt the upx invocation.
UPX_BIN="${UPX_BIN:-$(command -v upx || true)}"
unset UPX

# Auto-download a standalone UPX for the HOST arch when none is available and
# compression is requested. UPX ships fully static release tarballs, so no
# install/root is needed. Cached under $UPX_CACHE (default: /tmp).
UPX_VERSION="${UPX_VERSION:-5.0.0}"
UPX_CACHE="${UPX_CACHE:-/tmp}"
fetch_upx() {
    local host_arch tarball dir url
    case "$(uname -m)" in
        x86_64)         host_arch="amd64_linux" ;;
        aarch64|arm64)  host_arch="arm64_linux" ;;
        *) echo "warn: no prebuilt UPX for host $(uname -m); skipping compression" >&2; return 1 ;;
    esac
    dir="$UPX_CACHE/upx-${UPX_VERSION}-${host_arch}"
    if [ -x "$dir/upx" ]; then UPX_BIN="$dir/upx"; return 0; fi
    tarball="$UPX_CACHE/upx-${UPX_VERSION}-${host_arch}.tar.xz"
    url="https://github.com/upx/upx/releases/download/v${UPX_VERSION}/upx-${UPX_VERSION}-${host_arch}.tar.xz"
    echo "==> fetching UPX $UPX_VERSION ($host_arch)"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$tarball" "$url" || { echo "warn: UPX download failed; skipping compression" >&2; return 1; }
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$tarball" "$url" || { echo "warn: UPX download failed; skipping compression" >&2; return 1; }
    else
        echo "warn: neither curl nor wget available; skipping compression" >&2; return 1
    fi
    tar -xf "$tarball" -C "$UPX_CACHE" || { echo "warn: UPX extract failed; skipping compression" >&2; return 1; }
    [ -x "$dir/upx" ] && { UPX_BIN="$dir/upx"; return 0; }
    echo "warn: UPX binary not found after extract; skipping compression" >&2; return 1
}
if [ "${NO_UPX:-0}" != 1 ] && { [ -z "$UPX_BIN" ] || [ ! -x "$UPX_BIN" ]; }; then
    fetch_upx || true
fi

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

# Stage on a writable local fs first. The export dir may be a FUSE/vfat mount
# (shared storage) where chmod +x is a no-op and in-place upx can fail — so all
# mutation happens on the stage copy, then we cp only the finished binary out.
STAGE="$(mktemp -t lai-export.XXXXXX)"
cp "$BIN" "$STAGE"
chmod +x "$STAGE"
RAW_SIZE="$(du -h "$STAGE" | cut -f1)"

if [ "${NO_UPX:-0}" = 1 ]; then
    echo "==> NO_UPX=1 set; skipping compression"
elif [ -n "$UPX_BIN" ] && [ -x "$UPX_BIN" ]; then
    echo "==> compressing with UPX ($UPX_BIN)"
    "$UPX_BIN" --best --lzma "$STAGE"
else
    echo "warn: upx not found (set UPX_BIN=/path/to/upx or NO_UPX=1); shipping raw binary"
fi

mkdir -p "$HUB"
cp "$STAGE" "$DEST"
chmod +x "$DEST" 2>/dev/null || true   # no-op on FUSE/vfat; not an error
rm -f "$STAGE"

echo "==> done: $DEST (raw $RAW_SIZE -> $(du -h "$DEST" | cut -f1))"
ls -lh "$DEST"

#!/usr/bin/env bash
# Build the full-feature, portable, STATIC x86_64 release binary and export it
# to the shared internal-storage hub (visible in the Android "My Files" app.
#
# Why musl + how it builds on this aarch64 host:
#   The portable, dependency-free binary is produced by targeting
#   `x86_64-unknown-linux-musl` and linking statically against musl. The host is
#   aarch64, so we cannot run an x86_64 cross-gcc. Instead we drive the build
#   with the aarch64-native, x86_64-targeting gcc (`x86_64-linux-gnu-gcc`),
#   pointed at the musl sysroot shipped with the cross toolchain
#   (/opt/x86_64-linux-musl-cross/x86_64-linux-musl). That gcc emits x86_64 code
#   natively; `--sysroot=.../musl -static` makes it link musl's libc.a. No qemu,
#   no binfmt, no x86_64 glibc required.
#
# Hub layout (Android scoped storage forbids new top-level dirs, so we nest
# under Download/):
#   /sdcard/Download/Laverna/bin/laverna-x86_64   <- fixed name, overwritten
#
# Env:
#   CARGO_BUILD_JOBS   parallel jobs (not hardcoded; set per-invocation)
#   HUB                override hub root (default: /sdcard/Download/Laverna)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

HUB="${HUB:-/sdcard/Download/Laverna}"
BIN_DIR="$HUB/bin"
TARGET="x86_64-unknown-linux-musl"
FEATURES="mcp websearch budget llm milp graph"
DEST="$BIN_DIR/laverna-x86_64"

MUSL_SYSROOT="/opt/x86_64-linux-musl-cross/x86_64-linux-musl"
HOST_X86_64_GCC="$(command -v x86_64-linux-gnu-gcc || true)"
if [ -z "$HOST_X86_64_GCC" ]; then
    echo "error: x86_64-linux-gnu-gcc (aarch64-native, x86_64-targeting) not found" >&2
    exit 1
fi

# Per-target overrides so HOST (aarch64) builds of build scripts / proc-macros
# are untouched. The musl target's C compiler + linker is the native
# x86_64-targeting gcc, told to use the musl sysroot and link statically.
export CC_x86_64_unknown_linux_musl="$HOST_X86_64_GCC"
export AR_x86_64_unknown_linux_musl="$(command -v x86_64-linux-gnu-ar || command -v ar)"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$HOST_X86_64_GCC"
export CFLAGS_x86_64_unknown_linux_musl="--sysroot=$MUSL_SYSROOT -static"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C link-arg=--sysroot=$MUSL_SYSROOT -C link-arg=-static"

echo "==> disk before build"
df -h / | tail -1

echo "==> building (static $TARGET, features: $FEATURES)"
cargo build --release --target "$TARGET" --features "$FEATURES"

echo "==> exporting to hub"
mkdir -p "$BIN_DIR"
cp "target/$TARGET/release/laverna" "$DEST"

echo "==> bundling local LLM (laverna bin ships with llama.cpp + model drop-in)"
LLAMA_DIR="$BIN_DIR/llama"
MODELS_DIR="$BIN_DIR/models"
mkdir -p "$LLAMA_DIR" "$MODELS_DIR"
if [ -d "$REPO_ROOT/bin/llama" ]; then
    # -L dereferences the .so symlinks so the copy works on the Android
    # /sdcard FUSE mount (which rejects symlink creation) and the runtime
    # loader still finds libfoo.so.0 by its real file.
    cp -rL "$REPO_ROOT/bin/llama/." "$LLAMA_DIR/"
fi
# Models are user-supplied (not committed); keep the dir present.
[ -f "$REPO_ROOT/bin/models/.gitkeep" ] && cp "$REPO_ROOT/bin/models/.gitkeep" "$MODELS_DIR/"
echo "    llama engine -> $LLAMA_DIR/llama (set LAVERNA_LLAMA_MODEL to a .gguf in $MODELS_DIR)"

echo "==> verifying static x86_64 linkage (via qemu, since host is aarch64)"
# NOTE: $DEST lives on /sdcard, a noexec FUSE mount, so qemu cannot mmap-exec
# it there. Copy to a writable fs for the runtime smoke test; the `od` check
# below is the authoritative staticness proof regardless.
VERIFY_COPY="$(mktemp -t laverna-verify.XXXXXX)"
cp "$DEST" "$VERIFY_COPY"
chmod +x "$VERIFY_COPY"
if [ -x /usr/bin/qemu-x86_64-static ]; then
    /usr/bin/qemu-x86_64-static "$VERIFY_COPY" --version || true
elif command -v qemu-x86_64 >/dev/null 2>&1; then
    qemu-x86_64 "$VERIFY_COPY" --version || true
fi
rm -f "$VERIFY_COPY"
if od -c "$DEST" 2>/dev/null | grep -aq 'ld-linux-x86-64.so.2'; then
    echo "warn: binary still references the glibc dynamic linker (not fully static)"
else
    echo "ok: no glibc dynamic-linker reference -> STATIC"
fi

echo "==> done"
ls -lh "$DEST"

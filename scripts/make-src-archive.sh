#!/usr/bin/env bash
# Regenerate a clean L.ai source archive, excluding all build artifacts
# and prebuilt binaries. Reproducible: operates on the git tree, ignoring
# untracked/ignored files so the archive never contains artifacts.
#
# Usage: scripts/make-src-archive.sh [OUTPUT]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-${REPO_ROOT}/../downloads/lai-src-clean.tar.gz}"

cd "$REPO_ROOT"

# Use git archive so ignored/untracked artifacts (build/, .so, *.keystore,
# proof/bin/llama, athena-bin-release, etc.) are NEVER included.
git archive --format=tar.gz --prefix=lai/ -o "$OUT" HEAD

echo "Wrote clean source archive: $OUT"
echo "Size: $(du -h "$OUT" | cut -f1)"

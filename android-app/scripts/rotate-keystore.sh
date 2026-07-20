#!/usr/bin/env bash
# Rotate the L.ai Android signing key.
#
# SECURITY: the previously committed keystore (alias 'lai', password 'lai2026')
# is BURNED — treat any APK signed with it as compromisable. Generate a fresh
# keystore on a trusted machine with a JDK, then wire the secret into the
# gitignored `keystore.properties` (or env vars). Never commit the keystore or
# its password.
#
# Usage:
#   scripts/rotate-keystore.sh [output-keystore-path]
#     default output: android-app/lai-release.keystore
#
# The script:
#   1. generates a 4096-bit RSA key (alias 'lai', 10000-day validity),
#   2. prompts for a STRONG store/key password (reads from TTY, never echoed),
#   3. writes android-app/keystore.properties (gitignored) with the secret.
#
# After running: confirm `./gradlew assembleRelease` signs successfully, then
# delete any copy of the old keystore. APKs signed with the old key cannot be
# updated in place by the new key — existing installs must be uninstalled.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/app/lai-release.keystore}"
PROPS="$ROOT/app/keystore.properties"

command -v keytool >/dev/null 2>&1 || {
  echo "keytool not found — install a JDK (e.g. apt install openjdk-17-jdk)." >&2
  exit 1
}

[ -e "$OUT" ] && {
  echo "Refusing to overwrite existing keystore: $OUT" >&2
  echo "Move/delete it first if you intend to replace it." >&2
  exit 1
}

read -r -s -p "Enter a STRONG keystore password: " STORE_PW
echo
read -r -s -p "Confirm keystore password: " STORE_PW2
echo
[ "$STORE_PW" = "$STORE_PW2" ] || { echo "Passwords do not match." >&2; exit 1; }
[ "${#STORE_PW}" -ge 16 ] || { echo "Password too weak (need >=16 chars)." >&2; exit 1; }

keytool -genkeypair -v \
  -keystore "$OUT" \
  -keyalg RSA -keysize 4096 -validity 10000 -alias lai \
  -storepass "$STORE_PW" -keypass "$STORE_PW" \
  -dname "CN=L.ai, OU=Verify-Dont-Trust, O=L.ai, L=, ST=, C=US"

# Write the gitignored properties file. Permissions are restrictive; the value
# is written from the shell variable (not echoed to terminal/logs).
umask 077
{
  echo "storeFile=$(realpath --relative-to="$ROOT/app" "$OUT" 2>/dev/null || echo "$OUT")"
  echo "storePassword=$STORE_PW"
  echo "keyAlias=lai"
  echo "keyPassword=$STORE_PW"
} > "$PROPS"
umask 022

echo "Keystore written: $OUT"
echo "Properties written (gitignored): $PROPS"
echo "Verify with: ./gradlew assembleRelease"

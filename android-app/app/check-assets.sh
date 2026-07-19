#!/usr/bin/env bash
# Parse-check every bundled web asset. Exits nonzero on the first failure.
#
# This exists because the same stray `}` at the end of app.js shipped in two
# consecutive APKs. Gradle does not parse JavaScript, so `BUILD SUCCESSFUL`
# said nothing about whether the frontend could run. It could not.
set -uo pipefail

DIR="${1:-app/src/main/assets}"
fail=0

command -v node >/dev/null || { echo "check-assets: node not found — install it or this guard is decorative"; exit 127; }

for f in "$DIR"/*.js; do
  [ -e "$f" ] || continue
  if node --check "$f" 2>/tmp/asset-check.err; then
    echo "  ok    $f"
  else
    echo "  FAIL  $f"; sed 's/^/        /' /tmp/asset-check.err; fail=1
  fi
done

for f in "$DIR"/*.html; do
  [ -e "$f" ] || continue
  # Unclosed <script> or <div> won't stop a build either, but it will stop a boot.
  o=$(grep -o '<script' "$f" | wc -l); c=$(grep -o '</script>' "$f" | wc -l)
  if [ "$o" -ne "$c" ]; then echo "  FAIL  $f  ($o <script vs $c </script>)"; fail=1
  else echo "  ok    $f"; fi
done

[ "$fail" -eq 0 ] && echo "assets ok" || echo "assets FAILED — build would have shipped a non-booting app"
exit "$fail"

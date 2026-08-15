#!/usr/bin/env bash
# Tiny evaluation harness for the tri-state gate (Positioning doc, Stage 2).
#
# Runs every case in eval_tristate.tsv through `lai gate validate --format json`
# and asserts the emitted `tri_state` matches the expected value. Exits non-zero
# on the first mismatch so it can gate CI / a pre-push hook.
#
# Usage:
#   LAI=./target/debug/lai proof/scripts/eval_tristate.sh
#   (defaults to `lai` on PATH; set LAI to a specific binary)
set -uo pipefail

LAI="${LAI:-lai}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH="$SCRIPT_DIR/eval_tristate.tsv"

if ! command -v "$LAI" >/dev/null 2>&1; then
  echo "error: '$LAI' not found on PATH (set LAI=/path/to/lai)" >&2
  exit 2
fi

total=0
fail=0
while IFS=$'\t' read -r claim tool expect || [ -n "$claim" ]; do
  # Skip blank lines and comments.
  [ -z "$claim" ] && continue
  case "$claim" in \#*) continue ;; esac

  out="$("$LAI" gate validate "$claim" "$tool" --format json 2>/dev/null)" || {
    echo "ERROR  could not run gate validate for: $claim" >&2
    fail=$((fail + 1)); total=$((total + 1)); continue
  }
  got="$(printf '%s' "$out" | grep -o '"tri_state":"[a-z_]*"' | sed 's/.*:"//; s/"//')"
  total=$((total + 1))
  if [ "$got" = "$expect" ]; then
    printf 'PASS  [%-10s] %s\n' "$got" "$claim"
  else
    printf 'FAIL  expected %-10s got %-10s :: %s\n' "$expect" "$got" "$claim"
    fail=$((fail + 1))
  fi
done < "$BENCH"

echo
echo "tri-state eval: $total cases, $((total - fail)) passed, $fail failed"
[ "$fail" -eq 0 ]

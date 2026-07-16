#!/usr/bin/env bash
# List Laverna ticket files.
#
# Tickets are authored as Markdown files. The default source is Termux's private
# downloads dir, which proot reads *directly* — no Android per-app FUSE cache
# gap (unlike shared /sdcard, where files dropped from another app such as
# "My Files" may not appear until that FUSE mount's dir cache expires).
#
# Usage:
#   scripts/tickets.sh [--refresh]
#     --refresh   bust a stale FUSE dir cache before listing (only relevant when
#                 TICKETS_DIR points at shared /sdcard storage).
#
# Env:
#   TICKETS_DIR   where ticket *.md files live
#                 (default: $HOME/downloads, i.e. Termux ~/downloads)
set -euo pipefail

TICKETS_DIR="${TICKETS_DIR:-${HOME:-/data/data/com.termux/files/home}/downloads}"

if [ "${1:-}" = "--refresh" ]; then
  probe="$TICKETS_DIR/.cache-bust.$$"
  ( : > "$probe" && rm -f "$probe" ) 2>/dev/null || true
  sync 2>/dev/null || true
  ls "$TICKETS_DIR" >/dev/null 2>&1 || true
fi

if [ ! -d "$TICKETS_DIR" ]; then
  echo "ticket dir not found: $TICKETS_DIR" >&2
  exit 1
fi

shopt -s nullglob
files=("$TICKETS_DIR"/*.md)

if [ ${#files[@]} -eq 0 ]; then
  echo "no ticket files in $TICKETS_DIR"
  exit 0
fi

echo "tickets ($TICKETS_DIR):"
for f in "${files[@]}"; do
  title="$(grep -m1 '^# ' "$f" 2>/dev/null | sed 's/^# //')"
  printf '  %-45s %s\n' "$(basename "$f")" "${title:-<no title>}"
done

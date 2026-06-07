#!/usr/bin/env bash
# Build the `signatures` CLI in release mode and install it into a local bin dir.
#
# Installs:
#   - target/release/signatures  ->  $PREFIX/signatures   (default $PREFIX=~/.local/bin)
#
# Usage: scripts/install.sh [--prefix DIR] [--debug] [--dry-run]
#   --prefix DIR   install into DIR instead of ~/.local/bin (or set PREFIX env)
#   --debug        install the debug build (skips the release compile)
#   --dry-run, -n  show what would happen without building or copying
set -euo pipefail

# Resolve repo root (parent of this script's dir) so it works from anywhere.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

BIN="signatures"
PREFIX="${PREFIX:-$HOME/.local/bin}"
PROFILE="release"
DRY_RUN=0

while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) PREFIX="${2:?--prefix needs a directory}"; shift 2 ;;
    --prefix=*) PREFIX="${1#*=}"; shift ;;
    --debug) PROFILE="debug"; shift ;;
    --dry-run|-n) DRY_RUN=1; shift ;;
    -h|--help)
      sed -n '2,8p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) echo "install.sh: unknown argument: $1" >&2; exit 2 ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "install.sh: cargo not found on PATH" >&2
  exit 1
fi

SRC="target/$PROFILE/$BIN"
DEST="$PREFIX/$BIN"

# Build (unless installing an existing debug build).
if [ "$PROFILE" = "release" ]; then
  if [ "$DRY_RUN" -eq 1 ]; then
    echo "would run: cargo build --release"
  else
    echo "cargo build --release"
    cargo build --release
  fi
fi

if [ "$DRY_RUN" -eq 1 ]; then
  echo "would install: $SRC -> $DEST"
  exit 0
fi

if [ ! -x "$SRC" ]; then
  echo "install.sh: $SRC not found — build it first (omit --debug, or run 'cargo build')" >&2
  exit 1
fi

mkdir -p "$PREFIX"
install -m 0755 "$SRC" "$DEST"
echo "installed: $DEST"

# Friendly nudge if the install dir isn't on PATH.
case ":$PATH:" in
  *":$PREFIX:"*) ;;
  *) echo "note: $PREFIX is not on your PATH — add it, e.g.  export PATH=\"$PREFIX:\$PATH\"" ;;
esac

echo "install: done."

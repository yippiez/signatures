#!/usr/bin/env bash
# Clean build leftovers and throwaway scratch files from the signatures project.
#
# Removes:
#   - target/                  Cargo build output
#   - Cargo.lock               (only with --lock; normally committed for a binary crate)
#   - tests/fuzz/              workflow fuzzing scratch corpus
#   - **/*.rs.bk               rustfmt backups
#   - *.pdb                    debug symbols
#   - editor/OS noise          *.swp, *~, .DS_Store
#
# Usage: scripts/clean.sh [--lock] [--dry-run]
set -euo pipefail

# Resolve repo root (parent of this script's dir) so it works from anywhere.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

REMOVE_LOCK=0
DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --lock) REMOVE_LOCK=1 ;;
    --dry-run|-n) DRY_RUN=1 ;;
    -h|--help)
      sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) echo "clean.sh: unknown argument: $arg" >&2; exit 2 ;;
  esac
done

# rm() wrapper honoring --dry-run.
remove() {
  for path in "$@"; do
    [ -e "$path" ] || continue
    if [ "$DRY_RUN" -eq 1 ]; then
      echo "would remove: $path"
    else
      echo "removing: $path"
      rm -rf -- "$path"
    fi
  done
}

# Prefer `cargo clean` for build output when cargo is available (it also handles
# any custom target dir); fall back to removing target/ directly.
if [ "$DRY_RUN" -eq 0 ] && command -v cargo >/dev/null 2>&1 && [ -f Cargo.toml ]; then
  echo "cargo clean"
  cargo clean
else
  remove target
fi

remove tests/fuzz

# Pattern-based scratch files (null-delimited to handle odd names).
find . -type d -name target -prune -o \
  \( -name '*.rs.bk' -o -name '*.pdb' -o -name '*.swp' -o -name '*~' -o -name '.DS_Store' \) \
  -type f -print0 |
  while IFS= read -r -d '' f; do
    remove "$f"
  done

if [ "$REMOVE_LOCK" -eq 1 ]; then
  remove Cargo.lock
fi

echo "clean: done."

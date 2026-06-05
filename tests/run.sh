#!/usr/bin/env bash
#
# tests/run.sh — run the `signatures` CLI against every fixture under tests/
# and check its output, language by language.
#
# For each source file tests/<lang>/<name>.<ext> the runner runs
#   signatures --no-color <file>
# and compares stdout to a sibling snapshot tests/<lang>/<name>.<ext>.expected.
#
#   (no args)        check every fixture against its .expected snapshot
#   <lang> ...       only check those language folders (e.g. ./run.sh rust go)
#   --record, -r     (re)generate the .expected snapshots from current output
#   --bin PATH       use a specific signatures binary (default target/debug)
#   --help, -h       show this help
#
# A fixture with no .expected is a "smoke" check: it must not panic/crash, but
# its output is not compared. Any panic, or any mismatch in non-record mode,
# fails the run (exit 1).

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/debug/signatures"
RECORD=0
FILTERS=()

while [ $# -gt 0 ]; do
  case "$1" in
    -r|--record) RECORD=1 ;;
    --bin) shift; BIN="$1" ;;
    -h|--help) sed -n '2,22p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*) echo "unknown option: $1" >&2; exit 2 ;;
    *) FILTERS+=("$1") ;;
  esac
  shift
done

# Build the binary if it isn't there yet.
if [ ! -x "$BIN" ]; then
  echo "building signatures…" >&2
  cargo build --manifest-path "$ROOT/Cargo.toml" -q || { echo "build failed" >&2; exit 1; }
fi

# Colors (only when stdout is a terminal).
if [ -t 1 ]; then G=$'\e[32m'; R=$'\e[31m'; Y=$'\e[33m'; D=$'\e[2m'; Z=$'\e[0m'; else G= R= Y= D= Z=; fi

# Should this language folder be checked, given any filters?
want() {
  [ ${#FILTERS[@]} -eq 0 ] && return 0
  local lang="$1" f
  for f in "${FILTERS[@]}"; do [ "$f" = "$lang" ] && return 0; done
  return 1
}

pass=0; fail=0; smoke=0; rec=0
declare -a FAILED
tmp_out="$(mktemp)"; tmp_err="$(mktemp)"
trap 'rm -f "$tmp_out" "$tmp_err"' EXIT

# Source fixtures: everything under tests/<lang>/ except snapshots, notes and docs.
while IFS= read -r f; do
  rel="${f#"$ROOT"/}"
  lang="$(basename "$(dirname "$f")")"
  want "$lang" || continue

  "$BIN" --no-color "$f" >"$tmp_out" 2>"$tmp_err"
  if grep -q "panicked" "$tmp_err"; then
    echo "${R}CRASH${Z} $rel"
    sed 's/^/    /' "$tmp_err" | head -3
    fail=$((fail + 1)); FAILED+=("$rel (panic)"); continue
  fi

  exp="$f.expected"
  if [ "$RECORD" -eq 1 ]; then
    cp "$tmp_out" "$exp"; rec=$((rec + 1)); echo "${Y}REC${Z}   $rel"; continue
  fi

  if [ -f "$exp" ]; then
    if diff -u "$exp" "$tmp_out" >/dev/null 2>&1; then
      pass=$((pass + 1))
    else
      echo "${R}FAIL${Z}  $rel"
      diff -u "$exp" "$tmp_out" | sed -n '4,18p' | sed 's/^/    /'
      fail=$((fail + 1)); FAILED+=("$rel")
    fi
  else
    smoke=$((smoke + 1))
    echo "${D}smoke${Z} $rel ${D}(no .expected — ran clean)${Z}"
  fi
done < <(find "$ROOT/tests" -type f \
  ! -name '*.expected' ! -name '*.txt' ! -name '*.md' \
  ! -path "$ROOT/tests/run.sh" | sort)

echo
if [ "$RECORD" -eq 1 ]; then
  echo "${Y}recorded${Z} $rec snapshot(s)."
  exit 0
fi

echo "${G}pass${Z} $pass   ${R}fail${Z} $fail   ${D}smoke${Z} $smoke"
if [ "$fail" -gt 0 ]; then
  printf '  - %s\n' "${FAILED[@]}"
  exit 1
fi
exit 0

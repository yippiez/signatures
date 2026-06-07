#!/usr/bin/env bash
#
# tests/run.sh — run the `signatures` CLI against every fixture under tests/
# and check its output, language by language.
#
# Every fixture file lives directly under tests/ (no per-language subfolders),
# in one of two forms:
#
#   1. A merged case file  tests/<lang>.<ext>  holding many cases, each
#      introduced by a marker line  "@@CASE@@ <name>"  followed by that case's
#      source. The expected output lives in tests/<lang>.<ext>.expected using the
#      SAME markers. Each case is run in isolation (its own binary invocation),
#      so they cannot interfere — yet the file count never grows as cases are
#      added. This is the primary form (e.g. tests/rust.rs).
#
#   2. A loose fixture  tests/<name>.<ext>  with a sibling snapshot
#      tests/<name>.<ext>.expected, and NO @@CASE@@ markers. Kept for
#      byte-sensitive cases (e.g. a leading UTF-8 BOM) that a line-merged file
#      cannot represent faithfully (e.g. tests/rust_bom.rs).
#
# A file is treated as merged iff it contains a @@CASE@@ marker, else loose.
#
# For every case the runner runs  signatures --no-color <case>  and compares
# stdout to its expected snapshot.
#
#   (no args)        check every case against its expected snapshot
#   <lang> ...       only check those language folders (e.g. ./run.sh rust go)
#   --record, -r     (re)generate the expected snapshots from current output
#   --bin PATH       use a specific signatures binary (default target/debug)
#   --help, -h       show this help
#
# A case with no expected snapshot is a "smoke" check: it must not panic/crash,
# but its output is not compared. Any panic, or any mismatch in non-record mode,
# fails the run (exit 1).

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/debug/signatures"
MARKER='@@CASE@@ '
RECORD=0
FILTERS=()

while [ $# -gt 0 ]; do
  case "$1" in
    -r|--record) RECORD=1 ;;
    --bin) shift; BIN="$1" ;;
    -h|--help) sed -n '2,33p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
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

want() {
  [ ${#FILTERS[@]} -eq 0 ] && return 0
  local lang="$1" f
  for f in "${FILTERS[@]}"; do
    [ "$f" = "$lang" ] && return 0
    # A loose fixture named "<lang>_<suffix>" matches a "<lang>" filter.
    [ "${lang%%_*}" = "$f" ] && return 0
  done
  return 1
}

pass=0; fail=0; smoke=0; rec=0
declare -a FAILED
tmp_out="$(mktemp)"; tmp_err="$(mktemp)"
WORK="$(mktemp -d)"
trap 'rm -f "$tmp_out" "$tmp_err"; rm -rf "$WORK"' EXIT

# Run one case file and check it. $1=id (for reporting), $2=source file,
# $3=expected file ("" if none / smoke). Honors RECORD (writes $3).
check_case() {
  local id="$1" src="$2" exp="$3"
  "$BIN" --no-color "$src" >"$tmp_out" 2>"$tmp_err"
  if grep -q "panicked" "$tmp_err"; then
    echo "${R}CRASH${Z} $id"
    sed 's/^/    /' "$tmp_err" | head -3
    fail=$((fail + 1)); FAILED+=("$id (panic)"); return
  fi
  if [ "$RECORD" -eq 1 ]; then
    [ -n "$exp" ] && { cp "$tmp_out" "$exp"; rec=$((rec + 1)); }
    return
  fi
  if [ -n "$exp" ] && [ -f "$exp" ]; then
    if diff -u "$exp" "$tmp_out" >/dev/null 2>&1; then
      pass=$((pass + 1))
    else
      echo "${R}FAIL${Z}  $id"
      diff -u "$exp" "$tmp_out" | sed -n '4,18p' | sed 's/^/    /'
      fail=$((fail + 1)); FAILED+=("$id")
    fi
  else
    smoke=$((smoke + 1))
    echo "${D}smoke${Z} $id ${D}(no expected — ran clean)${Z}"
  fi
}

# Split a marker file ($1) into per-case files in dir ($2), named <case>.<sfx>
# ($3). Prints the ordered, original case names (one per line).
split_marker_file() {
  awk -v d="$2" -v sfx="$3" -v m="$MARKER" '
    index($0, m) == 1 {
      raw = substr($0, length(m) + 1); sub(/[ \t\r]+$/, "", raw)
      print raw
      san = raw; gsub(/[^A-Za-z0-9_.-]/, "_", san)
      cur = d "/" san "." sfx
      printf "" > cur
      next
    }
    cur { print > cur }
  ' "$1"
}

sanitize() { printf '%s' "$1" | sed 's/[^A-Za-z0-9_.-]/_/g'; }

# Process a merged cases.<ext> file: split, then check each case in isolation.
process_merged() {
  local src="$1" exp="${1}.expected" rel ext wd
  rel="${src#"$ROOT"/}"
  ext="${src##*.}"
  wd="$WORK/$(echo "$rel" | tr '/.' '__')"; mkdir -p "$wd"

  local -a names
  mapfile -t names < <(split_marker_file "$src" "$wd" "$ext")
  [ -f "$exp" ] && split_marker_file "$exp" "$wd" "expected" >/dev/null

  if [ "$RECORD" -eq 1 ]; then
    local out="$wd/.recorded"; : >"$out"
    local n san
    for n in "${names[@]}"; do
      san="$(sanitize "$n")"
      printf '%s%s\n' "$MARKER" "$n" >>"$out"
      "$BIN" --no-color "$wd/$san.$ext" 2>/dev/null >>"$out"
    done
    cp "$out" "$exp"; rec=$((rec + 1)); echo "${Y}REC${Z}   $rel (${#names[@]} cases)"
    return
  fi

  local n san expf
  for n in "${names[@]}"; do
    san="$(sanitize "$n")"
    expf="$wd/$san.expected"
    [ -f "$expf" ] || expf=""
    check_case "$rel::$n" "$wd/$san.$ext" "$expf"
  done
}

# Walk every fixture file directly under tests/.
while IFS= read -r f; do
  [ -e "$f" ] || continue
  base="$(basename "$f")"
  lang="${base%.*}"                                # strip extension
  want "$lang" || continue
  if grep -q "$MARKER" "$f" 2>/dev/null; then
    process_merged "$f"                            # has @@CASE@@ markers
  else
    rel="${f#"$ROOT"/}"
    exp="$f.expected"
    [ -f "$exp" ] || exp=""
    check_case "$rel" "$f" "$exp"                  # loose single-case fixture
  fi
done < <(find "$ROOT/tests" -maxdepth 1 -type f \
  ! -name '*.expected' ! -name '*.txt' ! -name '*.md' ! -name 'run.sh' | sort)

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

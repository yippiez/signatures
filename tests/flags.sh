#!/usr/bin/env bash
#
# tests/flags.sh — language-agnostic checker for the --format / --stream /
# --output flags. Enforces, across EVERY fixture under tests/, the invariants:
#
#   I1. Default (no new flags) output is byte-for-byte unchanged.
#       (We assert the default == plain+truncated here; tests/run.sh proves the
#        snapshot match. We do NOT modify run.sh.)
#   I2. For every format f in {plain,jsonl}: output(f) == output(f, --stream).
#   I3. In jsonl, every non-empty output line parses as valid JSON.
#   I4. jsonl record count == number of plain signature lines (a single-file
#       invocation prints no header, so plain lines == signatures).
#   I5. --output full never changes the NUMBER of lines vs truncated.
#
# Each fixture file is tested as a WHOLE invocation (these invariants are
# per-invocation, not per-case), so merged @@CASE@@ files are fine as-is.
#
# Exits non-zero and prints what failed on any violation. POSIX-ish bash; uses
# python3 only for JSON validation.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/debug/signatures"

if [ ! -x "$BIN" ]; then
  echo "building signatures…" >&2
  cargo build --manifest-path "$ROOT/Cargo.toml" -q || { echo "build failed" >&2; exit 1; }
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "flags.sh: python3 is required for JSON validation" >&2
  exit 2
fi

if [ -t 1 ]; then G=$'\e[32m'; R=$'\e[31m'; Z=$'\e[0m'; else G= R= Z=; fi

pass=0; fail=0
declare -a FAILED

# Always run with color forced off so plain output is deterministic; the binary
# also disables color when stdout is not a TTY, but --no-color is explicit.
NC=(--no-color)

# Validate that every non-empty line of stdin is valid JSON. Exit 0/1.
json_lines_ok() {
  python3 - "$@" <<'PY'
import sys, json
n = 0
for line in sys.stdin:
    s = line.rstrip("\n")
    if s == "":
        continue
    try:
        json.loads(s)
    except Exception as e:
        sys.stderr.write("invalid JSON line %d: %r (%s)\n" % (n + 1, s, e))
        sys.exit(1)
    n += 1
sys.exit(0)
PY
}

note_fail() { echo "${R}FAIL${Z}  $1"; [ -n "${2:-}" ] && printf '%s\n' "$2" | sed 's/^/    /'; fail=$((fail + 1)); FAILED+=("$1"); }
note_pass() { pass=$((pass + 1)); }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

check_fixture() {
  local f="$1" rel="${1#"$ROOT"/}"

  local def plain pstream jsonl jstream full

  def="$("$BIN" "${NC[@]}" "$f" 2>/dev/null)"
  plain="$("$BIN" "${NC[@]}" --format plain "$f" 2>/dev/null)"
  pstream="$("$BIN" "${NC[@]}" --format plain --stream "$f" 2>/dev/null)"
  jsonl="$("$BIN" "${NC[@]}" --format jsonl "$f" 2>/dev/null)"
  jstream="$("$BIN" "${NC[@]}" --format jsonl --stream "$f" 2>/dev/null)"
  full="$("$BIN" "${NC[@]}" --format plain --output full "$f" 2>/dev/null)"

  # I1: default == explicit plain+truncated.
  if [ "$def" != "$plain" ]; then
    note_fail "$rel I1 default!=plain" "$(diff <(printf '%s' "$def") <(printf '%s' "$plain") | head -6)"
    return
  fi

  # I2 (plain): stream byte-identical to buffered.
  if [ "$plain" != "$pstream" ]; then
    note_fail "$rel I2 plain stream mismatch" "$(diff <(printf '%s' "$plain") <(printf '%s' "$pstream") | head -6)"
    return
  fi
  # I2 (jsonl): stream byte-identical to buffered.
  if [ "$jsonl" != "$jstream" ]; then
    note_fail "$rel I2 jsonl stream mismatch" "$(diff <(printf '%s' "$jsonl") <(printf '%s' "$jstream") | head -6)"
    return
  fi

  # I3: every non-empty jsonl line is valid JSON.
  if ! printf '%s\n' "$jsonl" | json_lines_ok 2>"$tmp/jerr"; then
    note_fail "$rel I3 invalid JSON" "$(cat "$tmp/jerr")"
    return
  fi

  # Line counts. A single-file invocation prints no header in plain mode, so
  # plain lines == number of signatures. Guard empty strings (wc counts them).
  local plain_n jsonl_n full_n
  if [ -z "$plain" ]; then plain_n=0; else plain_n="$(printf '%s\n' "$plain" | grep -c '')"; fi
  if [ -z "$jsonl" ]; then jsonl_n=0; else jsonl_n="$(printf '%s\n' "$jsonl" | grep -c '')"; fi
  if [ -z "$full" ]; then full_n=0; else full_n="$(printf '%s\n' "$full" | grep -c '')"; fi

  # jsonl must never print blank separator lines: every line non-empty.
  local blanks
  blanks="$(printf '%s\n' "$jsonl" | grep -c '^$')"
  if [ -n "$jsonl" ] && [ "$blanks" -ne 0 ]; then
    note_fail "$rel jsonl has $blanks blank line(s)"
    return
  fi

  # I4: jsonl record count == plain signature line count.
  if [ "$jsonl_n" -ne "$plain_n" ]; then
    note_fail "$rel I4 count plain=$plain_n jsonl=$jsonl_n"
    return
  fi

  # I5: full has the same line count as truncated (plain).
  if [ "$full_n" -ne "$plain_n" ]; then
    note_fail "$rel I5 count truncated=$plain_n full=$full_n"
    return
  fi

  # jsonl must never contain ANSI escapes.
  if printf '%s' "$jsonl" | grep -q $'\e'; then
    note_fail "$rel jsonl contains ANSI escape"
    return
  fi

  note_pass
}

while IFS= read -r f; do
  [ -e "$f" ] || continue
  check_fixture "$f"
done < <(find "$ROOT/tests" -maxdepth 1 -type f \
  ! -name '*.expected' ! -name '*.txt' ! -name '*.md' \
  ! -name 'run.sh' ! -name 'flags.sh' | sort)

echo
echo "${G}pass${Z} $pass   ${R}fail${Z} $fail"
if [ "$fail" -gt 0 ]; then
  printf '  - %s\n' "${FAILED[@]}"
  exit 1
fi
exit 0

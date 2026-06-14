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
#   I5. --output full emits ONE record per TOP-LEVEL (indent 0) declaration:
#       the full-jsonl record count equals the number of indent-0 records in the
#       truncated jsonl. (Full mode prints whole bodies, so its plain line count
#       is unrelated to the truncated line count — we count records, not lines.)
#   I6. --output full streaming output is byte-identical to buffered, for both
#       plain and jsonl.
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

  local def plain pstream jsonl jstream full fpstream fjsonl fjstream

  def="$("$BIN" "${NC[@]}" "$f" 2>/dev/null)"
  plain="$("$BIN" "${NC[@]}" --format plain "$f" 2>/dev/null)"
  pstream="$("$BIN" "${NC[@]}" --format plain --stream "$f" 2>/dev/null)"
  jsonl="$("$BIN" "${NC[@]}" --format jsonl "$f" 2>/dev/null)"
  jstream="$("$BIN" "${NC[@]}" --format jsonl --stream "$f" 2>/dev/null)"
  full="$("$BIN" "${NC[@]}" --format plain --output full "$f" 2>/dev/null)"
  fpstream="$("$BIN" "${NC[@]}" --format plain --output full --stream "$f" 2>/dev/null)"
  fjsonl="$("$BIN" "${NC[@]}" --format jsonl --output full "$f" 2>/dev/null)"
  fjstream="$("$BIN" "${NC[@]}" --format jsonl --output full --stream "$f" 2>/dev/null)"

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
  local plain_n jsonl_n
  if [ -z "$plain" ]; then plain_n=0; else plain_n="$(printf '%s\n' "$plain" | grep -c '')"; fi
  if [ -z "$jsonl" ]; then jsonl_n=0; else jsonl_n="$(printf '%s\n' "$jsonl" | grep -c '')"; fi

  # Top-level (indent 0) record count in truncated jsonl, and full-jsonl record
  # count. Full mode emits, by coverage, the OUTERMOST declarations: every
  # top-level decl plus any decl whose enclosing block is itself not a printed
  # signature (e.g. a Rust `impl` method, a C# file-scoped namespace's members).
  # So the full count sits between the top-level count and the total count:
  #   top_n <= fjsonl_n <= jsonl_n.
  local top_n fjsonl_n
  if [ -z "$jsonl" ]; then top_n=0; else top_n="$(printf '%s\n' "$jsonl" | grep -c '"indent":0,')"; fi
  if [ -z "$fjsonl" ]; then fjsonl_n=0; else fjsonl_n="$(printf '%s\n' "$fjsonl" | grep -c '')"; fi

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

  # I5: full jsonl emits the outermost decls — at least every top-level decl,
  # at most every signature (it's a coverage subset, never a superset).
  if [ "$fjsonl_n" -lt "$top_n" ] || [ "$fjsonl_n" -gt "$jsonl_n" ]; then
    note_fail "$rel I5 full jsonl records=$fjsonl_n not in [top=$top_n, total=$jsonl_n]"
    return
  fi

  # I6: full-mode streaming is byte-identical to buffered (plain and jsonl).
  if [ "$full" != "$fpstream" ]; then
    note_fail "$rel I6 full plain stream mismatch" "$(diff <(printf '%s' "$full") <(printf '%s' "$fpstream") | head -6)"
    return
  fi
  if [ "$fjsonl" != "$fjstream" ]; then
    note_fail "$rel I6 full jsonl stream mismatch" "$(diff <(printf '%s' "$fjsonl") <(printf '%s' "$fjstream") | head -6)"
    return
  fi

  # Full jsonl must also be valid JSON and never contain ANSI escapes.
  if ! printf '%s\n' "$fjsonl" | json_lines_ok 2>"$tmp/jerr"; then
    note_fail "$rel I5 full jsonl invalid JSON" "$(cat "$tmp/jerr")"
    return
  fi
  if printf '%s' "$fjsonl" | grep -q $'\e'; then
    note_fail "$rel full jsonl contains ANSI escape"
    return
  fi

  # Full plain output must never contain ANSI escapes (raw source, no color).
  if printf '%s' "$full" | grep -q $'\e'; then
    note_fail "$rel full plain contains ANSI escape"
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

# ---------------------------------------------------------------------------
# Dedicated --output full content checks (the exact verbatim-body semantics).
# ---------------------------------------------------------------------------

# Assert that `--output full` on $2 (written to a temp .$1 file) prints exactly
# the expected text in $3.
expect_full() {
  local ext="$1" src="$2" want="$3" name="$4"
  local tf="$tmp/full_case.$ext"
  printf '%s' "$src" >"$tf"
  local got
  got="$("$BIN" "${NC[@]}" --output full "$tf" 2>/dev/null)"
  if [ "$got" = "$want" ]; then
    note_pass
  else
    note_fail "full-content: $name" "$(diff <(printf '%s' "$want") <(printf '%s' "$got") | head -20)"
  fi
}

# Case A: a multi-line function body is printed verbatim, in full, between two
# top-level decls separated by exactly one blank line. The non-constant
# assignment `lowercase` is omitted.
read -r -d '' CASE_A <<'PY'
import os

MAX = 5
lowercase = 1


def greet(name):
    # inner comment kept verbatim
    msg = "hi " + name
    return msg
PY
read -r -d '' WANT_A <<'PY'
MAX = 5

def greet(name):
    # inner comment kept verbatim
    msg = "hi " + name
    return msg
PY
expect_full py "$CASE_A"$'\n' "$WANT_A" "py multiline function body"

# Case B: a class with nested methods. The methods must NOT appear as their own
# top-level blocks — they appear ONLY inside the printed class body.
read -r -d '' CASE_B <<'PY'
class Greeter(Base):
    GREETING = "hi"

    def __init__(self, name):
        self.name = name

    def greet(self):
        return self.GREETING
PY
read -r -d '' WANT_B <<'PY'
class Greeter(Base):
    GREETING = "hi"

    def __init__(self, name):
        self.name = name

    def greet(self):
        return self.GREETING
PY
expect_full py "$CASE_B"$'\n' "$WANT_B" "py class with nested methods"

# Assert the nested methods appear exactly once (inside the class), not as
# separate top-level entries.
{
  tf="$tmp/full_b.py"; printf '%s\n' "$CASE_B" >"$tf"
  gotb="$("$BIN" "${NC[@]}" --output full "$tf" 2>/dev/null)"
  n_init="$(printf '%s\n' "$gotb" | grep -c 'def __init__(self, name):')"
  n_greet="$(printf '%s\n' "$gotb" | grep -c 'def greet(self):')"
  if [ "$n_init" -eq 1 ] && [ "$n_greet" -eq 1 ]; then
    note_pass
  else
    note_fail "full-content: nested methods appear once (init=$n_init greet=$n_greet)"
  fi
}

# Case C: a Rust `impl` block's methods are NOT emitted signatures' parents
# (impl is not itself a signature), so the coverage rule must still emit each
# method WITH its full body — otherwise every Rust method would vanish in full
# mode. Assert the method header and a body line both appear.
read -r -d '' CASE_C <<'RS'
struct Point {
    x: i32,
}

impl Point {
    fn new(x: i32) -> Self {
        Point { x }
    }
}

fn main() {
    let _ = Point::new(1);
}
RS
{
  tf="$tmp/full_c.rs"; printf '%s\n' "$CASE_C" >"$tf"
  gotc="$("$BIN" "${NC[@]}" --output full "$tf" 2>/dev/null)"
  if printf '%s\n' "$gotc" | grep -q 'fn new(x: i32) -> Self' \
     && printf '%s\n' "$gotc" | grep -q 'Point { x }' \
     && printf '%s\n' "$gotc" | grep -q 'fn main()'; then
    note_pass
  else
    note_fail "full-content: rust impl method body present" "$gotc"
  fi
}

echo
echo "${G}pass${Z} $pass   ${R}fail${Z} $fail"
if [ "$fail" -gt 0 ]; then
  printf '  - %s\n' "${FAILED[@]}"
  exit 1
fi
exit 0

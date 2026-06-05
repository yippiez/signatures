#!/bin/bash

# This file tests that fake declarations inside comments/strings are ignored.

REAL_CONST="actual"
readonly REAL_READONLY="yes"

# function fake_in_comment() { echo "not real"; }
# FAKE_CONST=123
# readonly FAKE_READONLY="nope"

real_function() {
  # Inside comment: function inner_fake() { true; }
  # INNER_FAKE=999
  echo "real"
}

function string_test() {
  local msg="function not_a_function() { echo hi; }"
  local msg2='readonly NOT_CONST=42'
  echo "$msg $msg2"
}

function heredoc_text() {
  cat <<'EOF'
This heredoc contains prose, not code.
It mentions concepts like "function" and "readonly" in plain English.
No actual shell syntax that looks like a declaration on its own line.
EOF
}

function double_quoted_heredoc() {
  cat <<"MARKER"
Just some text output.
Nothing that resembles a shell function or constant.
MARKER
}

real_at_end() {
  echo "I am real"
}

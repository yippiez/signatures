@@CASE@@ comments_strings
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
@@CASE@@ constants
#!/bin/bash

# Constants and variable declarations — only UPPER_CASE and readonly are signatures

readonly MAX_CONNECTIONS=100
readonly MIN_TIMEOUT=5
readonly APP_ROOT="/opt/app"
readonly DB_HOST="localhost"
readonly PI=3.14159

ENVIRONMENT="production"
LOG_LEVEL="INFO"
RETRY_COUNT=3
BUFFER_SIZE=4096

# lowercase vars — should NOT appear as signatures
name="world"
count=0
tmp_dir="/tmp"
local_flag=false

get_max() {
  echo "$MAX_CONNECTIONS"
}

function get_env() {
  echo "$ENVIRONMENT"
}

# Mixed assignment — only readonly triggers
readonly COMPILED_REGEX="^[a-z]+$"
DEBUG_MODE=0

set_debug() {
  DEBUG_MODE=1
}

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly VERSION_STRING="1.0.0-beta"
@@CASE@@ edge
#!/bin/bash

# Edge cases: malformed but parseable constructs

readonly EMPTY_VAL=
readonly QUOTED_SPACES="hello world"
UPPER_NO_READONLY=99
ANOTHER_UPPER=true

# Function with unusual spacing
function   spaced_name   () {
  echo "spaced"
}

no_space_brace(){ echo "no space"; }

function one_liner() { echo "one liner"; }

# Numeric-ish names
func123() {
  echo "numeric suffix"
}

function _private_func() {
  echo "underscore prefix"
}

__double_under() {
  echo "double underscore"
}

function UPPER_FUNC() {
  echo "uppercase function name"
}

# readonly with no value
readonly DECLARED_ONLY

# Multiple readonly on one logical block (separate lines)
readonly A_CONST=1
readonly B_CONST=2
readonly C_CONST=3

normal_func() {
  echo "normal"
}

# Trailing content after closing brace on same line is unusual but handled
function compact() { true; }

LAST_CONST="end"
@@CASE@@ functions
#!/bin/bash

# Demonstrates both function definition styles exhaustively

function alpha() {
  echo "alpha"
}

function beta() {
  echo "beta"
}

gamma() {
  echo "gamma"
}

delta() {
  echo "delta"
}

function epsilon() {
  true
}

zeta() {
  true
}

function with_args() {
  local x="$1"
  local y="$2"
  echo "$x $y"
}

no_body_call() {
  :
}

function multiline_body() {
  local a=1
  local b=2
  local c=$((a + b))
  echo "$c"
}

# A function with a long name
function this_is_a_very_long_function_name_for_testing() {
  echo "long name"
}

short() {
  echo "short"
}
@@CASE@@ heredoc_at_toplevel
cat <<'EOF'
FAKE_CONST=100
fake_func() {
  :
}
EOF
REAL=1
@@CASE@@ heredoc_in_function
myfunc() {
  cat <<'EOF'
fake() {
  echo inside heredoc
}
EOF
}
@@CASE@@ nested
#!/bin/bash

# Functions defined inside conditionals, loops, and case statements

PLATFORM="linux"
readonly CONFIG_FILE="/etc/app.conf"

if [ "$PLATFORM" = "linux" ]; then
  function linux_setup() {
    echo "linux"
  }
else
  function fallback_setup() {
    echo "fallback"
  }
fi

for module in core net io; do
  eval "load_${module}() { echo loading ${module}; }"
done

case "$PLATFORM" in
  linux)
    platform_name() { echo "Linux"; }
    ;;
  darwin)
    platform_name() { echo "Darwin"; }
    ;;
esac

outer() {
  inner_helper() {
    echo "inner"
  }
  inner_helper
}

function conditional_func() {
  if true; then
    nested_if() { echo "nested if body"; }
  fi
}

while_wrapper() {
  local i=0
  while [ "$i" -lt 1 ]; do
    function while_inner() { echo "while inner"; }
    i=$((i + 1))
  done
}

TOP_LEVEL_AFTER=42

top_after_nested() {
  echo "top level after nested defs"
}
@@CASE@@ realworld
#!/bin/bash

# Deployment script for a web service

APP_NAME="myapp"
readonly VERSION="2.1.0"
readonly LOG_DIR="/var/log/myapp"
BUILD_DIR="/tmp/build"
MAX_RETRIES=3
TIMEOUT=30

check_dependencies() {
  command -v docker >/dev/null 2>&1 || { echo "docker required"; exit 1; }
  command -v curl >/dev/null 2>&1 || { echo "curl required"; exit 1; }
}

function build_image() {
  docker build -t "${APP_NAME}:${VERSION}" .
}

function push_image() {
  docker push "${APP_NAME}:${VERSION}"
}

wait_for_health() {
  local url="$1"
  local retries=0
  while [ "$retries" -lt "$MAX_RETRIES" ]; do
    curl -sf "$url/health" && return 0
    retries=$((retries + 1))
    sleep 5
  done
  return 1
}

function deploy_service() {
  build_image
  push_image
  docker service update --image "${APP_NAME}:${VERSION}" "${APP_NAME}"
  wait_for_health "http://localhost:8080"
}

rollback() {
  local prev_version="$1"
  docker service update --image "${APP_NAME}:${prev_version}" "${APP_NAME}"
}

function cleanup() {
  rm -rf "$BUILD_DIR"
  docker image prune -f
}
@@CASE@@ sample
#!/bin/bash

MAX=10
readonly NAME="app"

# function not_real() in a comment

greet() {
  echo "hi"
}

function deploy() {
  echo "deploying"
}
@@CASE@@ subshell_and_arith_bodies
outer() (
  fake_inner() { echo "false"; }
)
REAL="yes"
arith_func() (( x = 1 + 2 ))
real_func() { echo "real"; }
@@CASE@@ multiline_constant_span
SQL=\
  "SELECT * FROM t\
   WHERE id = 1"
GREETING="Hello,\
 World"
function after() {
  echo "after"
}

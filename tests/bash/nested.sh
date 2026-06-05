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

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

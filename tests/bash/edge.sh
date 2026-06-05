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

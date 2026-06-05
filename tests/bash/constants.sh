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

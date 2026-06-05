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

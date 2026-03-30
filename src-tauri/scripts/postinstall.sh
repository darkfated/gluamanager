#!/bin/sh
set -eu

BIN_NAME="gluamanager"
LINK_PATH="/usr/local/bin/${BIN_NAME}"

find_installed_binary() {
  find /opt /usr/lib -type f -name "${BIN_NAME}" -perm -111 2>/dev/null | head -n 1
}

TARGET_PATH="$(find_installed_binary || true)"

if [ -z "${TARGET_PATH}" ]; then
  exit 0
fi

mkdir -p /usr/local/bin
ln -sfn "${TARGET_PATH}" "${LINK_PATH}"

#!/bin/sh
set -eu

BIN_NAME="gluamanager"
LINK_PATH="/usr/local/bin/${BIN_NAME}"

if [ -L "${LINK_PATH}" ] && [ "$(readlink -f "${LINK_PATH}" 2>/dev/null || true)" != "" ]; then
  TARGET_PATH="$(readlink -f "${LINK_PATH}")"
  case "${TARGET_PATH}" in
    /opt/*|/usr/lib/*)
      rm -f "${LINK_PATH}"
      ;;
  esac
fi

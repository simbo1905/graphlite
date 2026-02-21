#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROCKS_TREE="${SCRIPT_DIR}/.luarocks"

check_lua() {
  if ! command -v lua >/dev/null 2>&1; then
    echo "ERROR: lua is not installed or not in PATH."
    echo "Install Lua 5.4+ before running this setup script."
    exit 1
  fi

  local version_output
  version_output="$(lua -v 2>&1 || true)"

  local major minor
  major="$(echo "${version_output}" | sed -n 's/^Lua \([0-9][0-9]*\)\.\([0-9][0-9]*\).*$/\1/p')"
  minor="$(echo "${version_output}" | sed -n 's/^Lua \([0-9][0-9]*\)\.\([0-9][0-9]*\).*$/\2/p')"

  if [[ -z "${major}" || -z "${minor}" ]]; then
    echo "ERROR: unable to parse Lua version from: ${version_output}"
    exit 1
  fi

  if (( major < 5 || (major == 5 && minor < 4) )); then
    echo "ERROR: Lua ${major}.${minor} detected. Lua 5.4+ is required for setup."
    exit 1
  fi
}

check_luarocks() {
  if ! command -v luarocks >/dev/null 2>&1; then
    echo "ERROR: luarocks is not installed or not in PATH."
    echo "Install LuaRocks, then re-run this script."
    exit 1
  fi
}

install_dkjson() {
  mkdir -p "${ROCKS_TREE}"
  echo "Installing dkjson into ${ROCKS_TREE} ..."
  luarocks --tree "${ROCKS_TREE}" install dkjson
}

main() {
  check_lua
  check_luarocks
  install_dkjson

  cat <<EOF
Setup complete.

dkjson is installed in:
  ${ROCKS_TREE}

The Lua examples auto-add this local LuaRocks tree at runtime.

Next:
  1) Build GraphLite FFI:
       cargo build --release -p graphlite-ffi
  2) Run examples:
       cd examples/lua/sdk
       luajit drug_discovery.lua
EOF
}

main "$@"

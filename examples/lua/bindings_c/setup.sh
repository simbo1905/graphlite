#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROCKS_TREE="${SCRIPT_DIR}/.rocks"

require_cmd() {
    local cmd="$1"
    local message="$2"
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        echo "ERROR: ${message}" >&2
        exit 1
    fi
}

pick_lua_bin() {
    if command -v lua5.4 >/dev/null 2>&1; then
        echo "lua5.4"
        return 0
    fi

    if command -v lua >/dev/null 2>&1; then
        echo "lua"
        return 0
    fi

    return 1
}

require_min_lua_54() {
    local lua_bin="$1"
    local major minor

    read -r major minor < <("${lua_bin}" -e 'local M,m=_VERSION:match("Lua (%d+)%.(%d+)"); if not M then os.exit(1) end; print(M,m)')

    if [[ -z "${major:-}" || -z "${minor:-}" ]]; then
        echo "ERROR: could not determine Lua version from ${lua_bin}" >&2
        exit 1
    fi

    if (( major < 5 || (major == 5 && minor < 4) )); then
        echo "ERROR: Lua >= 5.4 is required (found ${major}.${minor})" >&2
        exit 1
    fi
}

main() {
    local lua_bin

    lua_bin="$(pick_lua_bin || true)"
    if [[ -z "${lua_bin}" ]]; then
        echo "ERROR: Lua is not installed. Please install Lua 5.4+." >&2
        exit 1
    fi

    require_min_lua_54 "${lua_bin}"
    require_cmd "luarocks" "luarocks is not installed. Please install luarocks."

    echo "Using Lua interpreter: ${lua_bin} ($("${lua_bin}" -e 'print(_VERSION)'))"
    echo "Installing dkjson into ${ROCKS_TREE} ..."

    luarocks --lua-version=5.4 --tree "${ROCKS_TREE}" install dkjson

    echo "dkjson installed in local .rocks tree."
    echo "Next: build graphlite_lua module and run ${lua_bin} basic_usage.lua"
}

main "$@"

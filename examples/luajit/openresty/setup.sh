#!/usr/bin/env sh
set -eu

ROOT_DIR=$(cd "$(dirname "$0")" && pwd)

find_repo_root() {
  dir="$1"
  while [ "$dir" != "/" ]; do
    if [ -d "$dir/.git" ]; then
      echo "$dir"
      return 0
    fi
    dir=$(dirname "$dir")
  done
  return 1
}

REPO_ROOT=$(find_repo_root "$ROOT_DIR")
if [ -z "${REPO_ROOT:-}" ]; then
  echo "repo root not found (no .git directory)"
  exit 1
fi

RUNTIME_DIR="$REPO_ROOT/graphlite-ffi/data/luajit"
DB_DIR="$RUNTIME_DIR/db"

if ! command -v luajit >/dev/null 2>&1; then
	echo "luajit not found in PATH"
	exit 1
fi

if ! luajit -v 2>&1 | grep -q "LuaJIT"; then
	echo "luajit is not LuaJIT"
	exit 1
fi

mkdir -p "$RUNTIME_DIR" "$DB_DIR" "$RUNTIME_DIR/logs"

if [ -z "${GRAPHLITE_LIB:-}" ]; then
	case "$(uname -s)" in
	Darwin)
		LIB_NAME="libgraphlite_ffi.dylib"
		;;
	*)
		LIB_NAME="libgraphlite_ffi.so"
		;;
	esac
	GRAPHLITE_LIB="$ROOT_DIR/../../../target/release/$LIB_NAME"
fi

if [ ! -f "$GRAPHLITE_LIB" ]; then
	echo "GraphLite FFI library not found at: $GRAPHLITE_LIB"
	echo "Build it first: cargo build --release -p graphlite-ffi"
	exit 1
fi

export REPO_ROOT
export GRAPHLITE_LIB
export GRAPHLITE_DB_PATH="$DB_DIR"

luajit "$ROOT_DIR/setup.lua"

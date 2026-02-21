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
CONF_TEMPLATE="$ROOT_DIR/nginx.conf.in"
CONF_OUT="$RUNTIME_DIR/nginx.conf"
PORT_FILE="$RUNTIME_DIR/port.txt"

if ! command -v openresty >/dev/null 2>&1; then
	echo "openresty not found in PATH"
	exit 1
fi

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

if [ ! -d "$DB_DIR" ]; then
	echo "Database not found. Run ./setup.sh first."
	exit 1
fi

if ! command -v luajit >/dev/null 2>&1; then
	echo "luajit not found in PATH"
	exit 1
fi

if ! luajit -v 2>&1 | grep -q "LuaJIT"; then
	echo "luajit is not LuaJIT"
	exit 1
fi

export REPO_ROOT
export GRAPHLITE_LIB
export GRAPHLITE_DB_PATH="$DB_DIR"

if ! luajit "$ROOT_DIR/setup.lua" check >/dev/null 2>&1; then
	echo "Database check failed. Run ./setup.sh first."
	exit 1
fi

VERSION=$(openresty -v 2>&1 | sed -n 's/^nginx version: openresty\///p')
MIN_VERSION="1.21.0.0"

version_ge() {
	a=$1
	b=$2
	IFS=.
	# shellcheck disable=SC2086
	set -- $a
	a1=${1:-0} a2=${2:-0} a3=${3:-0} a4=${4:-0}
	# shellcheck disable=SC2086
	set -- $b
	b1=${1:-0} b2=${2:-0} b3=${3:-0} b4=${4:-0}

	if [ "$a1" -gt "$b1" ]; then return 0; fi
	if [ "$a1" -lt "$b1" ]; then return 1; fi
	if [ "$a2" -gt "$b2" ]; then return 0; fi
	if [ "$a2" -lt "$b2" ]; then return 1; fi
	if [ "$a3" -gt "$b3" ]; then return 0; fi
	if [ "$a3" -lt "$b3" ]; then return 1; fi
	if [ "$a4" -ge "$b4" ]; then return 0; fi
	return 1
}

if ! version_ge "$VERSION" "$MIN_VERSION"; then
	echo "openresty $VERSION is below required $MIN_VERSION"
	exit 1
fi

mkdir -p "$RUNTIME_DIR" "$RUNTIME_DIR/logs"

pick_port() {
	if command -v lsof >/dev/null 2>&1; then
		i=0
		while [ "$i" -lt 20 ]; do
			PORT=$(awk 'BEGIN{srand(); print int(49152 + rand() * 16383)}')
			if ! lsof -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
				echo "$PORT"
				return
			fi
			i=$((i + 1))
		done
	fi
	echo "49152"
}

PORT=$(pick_port)

sed -e "s|__PORT__|$PORT|g" \
	-e "s|__ROOT__|$ROOT_DIR|g" \
	-e "s|__RUNTIME__|$RUNTIME_DIR|g" \
	"$CONF_TEMPLATE" >"$CONF_OUT"

openresty -p "$RUNTIME_DIR" -c "$CONF_OUT"

echo "$PORT" >"$PORT_FILE"

echo "OpenResty running on http://127.0.0.1:$PORT"

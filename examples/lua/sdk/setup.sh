#!/bin/bash
# Setup script for GraphLite Lua examples.
# Checks: Lua 5.4+ (or LuaJIT), luarocks, installs dkjson.
# Run from examples/lua/sdk/: ./setup.sh

set -e

LUA_CMD=""
LUA_VERSION=""

# Require Lua 5.4+ or LuaJIT (SDK uses FFI, LuaJIT preferred)
if command -v luajit &>/dev/null; then
  LUA_CMD="luajit"
  LUA_VERSION=$(luajit -v 2>&1 || true)
  echo "Found LuaJIT: $LUA_VERSION"
elif command -v lua5.4 &>/dev/null; then
  LUA_CMD="lua5.4"
  LUA_VERSION=$($LUA_CMD -v 2>&1 || true)
  echo "Found Lua: $LUA_VERSION"
elif command -v lua &>/dev/null; then
  LUA_CMD="lua"
  LUA_VERSION=$($LUA_CMD -v 2>&1 || true)
  echo "Found Lua: $LUA_VERSION"
  if ! echo "$LUA_VERSION" | grep -qE "Lua (5\.[4-9]|[6-9])"; then
    echo "ERROR: Lua 5.4+ required (found: $LUA_VERSION)"
    exit 1
  fi
else
  echo "ERROR: Lua 5.4+ or LuaJIT not found."
  echo "  Ubuntu: sudo apt install lua5.4  OR  sudo apt install luajit"
  echo "  macOS:  brew install lua  OR  brew install luajit"
  exit 1
fi

if ! command -v luarocks &>/dev/null; then
  echo "ERROR: luarocks not found. Install it first:"
  echo "  Ubuntu: sudo apt install luarocks"
  echo "  macOS:  brew install luarocks"
  exit 1
fi
echo "Found luarocks: $(luarocks --version 2>&1 | head -1)"

echo "Installing dkjson..."
luarocks install dkjson --local

echo ""
echo "Setup complete. To ensure dkjson is on package.path, run:"
echo "  eval \$(luarocks path --bin)"
echo "Then run examples with:"
echo "  $LUA_CMD drug_discovery.lua"
echo "  $LUA_CMD basic_usage.lua"

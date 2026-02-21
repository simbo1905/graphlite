#!/usr/bin/env bash
# GraphLite Lua SDK Examples — prerequisites setup
# Checks that Lua >= 5.4 and luarocks are installed, then installs dkjson.
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

ok()   { echo -e "  ${GREEN}✓${NC} $*"; }
fail() { echo -e "  ${RED}✗${NC} $*"; exit 1; }

echo "=== GraphLite Lua SDK Examples — Setup ==="
echo

# --- 1. Check Lua -----------------------------------------------------------
echo "Checking Lua..."
if ! command -v lua &>/dev/null; then
  fail "lua not found. Please install Lua 5.4+:
       Ubuntu/Debian : sudo apt-get install lua5.4
       macOS         : brew install lua"
fi

LUA_VERSION=$(lua -e 'print(_VERSION:match("%d+%.%d+"))' 2>/dev/null || true)
if [ -z "$LUA_VERSION" ]; then
  LUA_VERSION=$(lua -v 2>&1 | grep -oE '[0-9]+\.[0-9]+' | head -1)
fi

MAJOR=$(echo "$LUA_VERSION" | cut -d. -f1)
MINOR=$(echo "$LUA_VERSION" | cut -d. -f2)

if [ -z "$MAJOR" ] || [ -z "$MINOR" ]; then
  fail "Could not determine Lua version."
fi

if [ "$MAJOR" -lt 5 ] || { [ "$MAJOR" -eq 5 ] && [ "$MINOR" -lt 4 ]; }; then
  fail "Lua $LUA_VERSION found, but 5.4+ is required.
       Ubuntu/Debian : sudo apt-get install lua5.4
       macOS         : brew install lua"
fi
ok "Lua $LUA_VERSION"

# --- 2. Check luarocks ------------------------------------------------------
echo "Checking luarocks..."
if ! command -v luarocks &>/dev/null; then
  fail "luarocks not found. Please install it:
       Ubuntu/Debian : sudo apt-get install luarocks
       macOS         : brew install luarocks
       Other         : https://luarocks.org/#quick-start"
fi
ok "luarocks $(luarocks --version 2>&1 | head -1)"

# --- 3. Install dkjson ------------------------------------------------------
echo "Installing dkjson..."
if luarocks show dkjson &>/dev/null; then
  ok "dkjson already installed"
else
  luarocks install dkjson
  ok "dkjson installed"
fi

echo
echo "=== Setup complete ==="
echo "You can now run the examples, e.g.:  lua drug_discovery.lua"

#!/usr/bin/env bash
# setup.sh -- Check prerequisites and install Lua dependencies for the
#              GraphLite Lua 5.4 basic-usage demo.
#
# Usage:  ./setup.sh
#
# What it does:
#   1. Verifies lua5.4 (or lua >= 5.4) is installed.
#   2. Verifies luarocks is installed.
#   3. Installs dkjson via luarocks (user-local, for Lua 5.4).

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'  # No colour

ok()   { printf "${GREEN}[OK]${NC} %s\n" "$*"; }
fail() { printf "${RED}[FAIL]${NC} %s\n" "$*"; exit 1; }

# ---------------------------------------------------------------
# 1. Check Lua >= 5.4
# ---------------------------------------------------------------
LUA_CMD=""
for candidate in lua5.4 lua54 lua; do
    if command -v "$candidate" >/dev/null 2>&1; then
        LUA_CMD="$candidate"
        break
    fi
done

if [ -z "$LUA_CMD" ]; then
    fail "Lua interpreter not found.  Install Lua 5.4:
      Debian/Ubuntu:  sudo apt install lua5.4
      macOS:          brew install lua@5.4
      Fedora:         sudo dnf install lua"
fi

LUA_VER=$("$LUA_CMD" -v 2>&1 | grep -oP '\d+\.\d+' | head -1)
LUA_MAJOR=$(echo "$LUA_VER" | cut -d. -f1)
LUA_MINOR=$(echo "$LUA_VER" | cut -d. -f2)

if [ "$LUA_MAJOR" -lt 5 ] || { [ "$LUA_MAJOR" -eq 5 ] && [ "$LUA_MINOR" -lt 4 ]; }; then
    fail "Lua $LUA_VER detected, but >= 5.4 is required."
fi
ok "Lua $LUA_VER found ($LUA_CMD)"

# ---------------------------------------------------------------
# 2. Check luarocks
# ---------------------------------------------------------------
if ! command -v luarocks >/dev/null 2>&1; then
    fail "luarocks not found.  Install it:
      Debian/Ubuntu:  sudo apt install luarocks
      macOS:          brew install luarocks
      Fedora:         sudo dnf install luarocks"
fi
ROCKS_VER=$(luarocks --version 2>&1 | head -1)
ok "luarocks found ($ROCKS_VER)"

# ---------------------------------------------------------------
# 3. Install dkjson via luarocks (targeting Lua 5.4)
# ---------------------------------------------------------------
echo ""
echo "Installing dkjson for Lua 5.4 ..."
if luarocks --lua-version 5.4 install --local dkjson 2>&1; then
    ok "dkjson installed (user-local, Lua 5.4)"
else
    echo ""
    echo "User-local install failed; trying system-wide ..."
    if sudo luarocks --lua-version 5.4 install dkjson 2>&1; then
        ok "dkjson installed (system-wide, Lua 5.4)"
    else
        fail "Could not install dkjson via luarocks."
    fi
fi

echo ""
echo "To make luarocks packages visible to lua5.4, run:"
echo '  eval "$(luarocks --lua-version 5.4 path)"'
echo ""
ok "Setup complete.  You can now run:  make && make run"

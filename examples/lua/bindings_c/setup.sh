#!/bin/sh
set -e

echo "Checking for Lua..."
if command -v lua5.4 >/dev/null 2>&1; then
	LUA_CMD="lua5.4"
elif command -v lua >/dev/null 2>&1; then
	LUA_CMD="lua"
else
	echo "Error: lua5.4 is required but not installed."
	exit 1
fi

LUA_VERSION=$($LUA_CMD -e 'print(_VERSION)' | awk '{print $2}')
if [ "$LUA_VERSION" != "5.4" ]; then
	echo "Error: Lua 5.4 is required, but found $LUA_VERSION"
	exit 1
fi
echo "Found Lua 5.4."

echo "Checking for luarocks..."
if ! command -v luarocks >/dev/null 2>&1; then
	echo "Error: luarocks is required but not installed."
	exit 1
fi
echo "Found luarocks."

echo "Installing dkjson locally into lua_modules/..."
luarocks install dkjson --tree=lua_modules

echo "Setup complete! You can now run 'make run'"

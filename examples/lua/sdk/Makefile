.PHONY: lint check setup clean all

all: lint

# Lint: runs luacheck on all Lua files in this directory.
# Uses .luacheckrc for configuration.
# Exit code is non-zero if ANY warning or error is found — the build fails.
lint:
	luacheck *.lua --no-color --codes

# Alias
check: lint

# First-time setup: validate Lua >= 5.4, luarocks, install dkjson
setup:
	./setup.sh

clean:
	rm -rf ./drug_discovery_lua_sdk_db ./basic_usage_lua_sdk_db

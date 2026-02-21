.PHONY: lint check test clean all

all: lint

# Lint: runs luacheck with --codes so warning IDs are visible.
# Uses .luacheckrc for configuration.
# Exit code is non-zero if ANY warning or error is found — the build fails.
lint:
	luacheck src/ tests/ --no-color --codes

# Alias
check: lint

# Run the smoke test (requires the FFI library to be built and on LD_LIBRARY_PATH)
test:
	lua tests/smoke_test.lua

clean:
	rm -rf /tmp/graphlite_lua_sdk_smoke_*

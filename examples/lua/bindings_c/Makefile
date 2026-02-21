# GraphLite Lua 5.4 C Module - Makefile
#
# Prerequisites:
#   1. Build Rust FFI: cargo build --release -p graphlite-ffi
#   2. Lua 5.4 dev headers and library (liblua5.4-dev on Debian/Ubuntu)
#
# Usage:
#   make          - build graphlite_lua.so (or .dylib on macOS)
#   make run      - run basic_usage.lua
#   make clean    - remove build artifacts

# Paths relative to this Makefile (examples/lua/bindings_c/)
ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST)))/../../..)
TARGET_RELEASE := $(ROOT)/target/release
# graphlite.h is bundled here; alternatively use -I$(ROOT)/graphlite-ffi if built

# Platform detection
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
  MODULE := graphlite_lua.so
  FFI_LIB := $(TARGET_RELEASE)/libgraphlite_ffi.dylib
  LDFLAGS_EXTRA := -undefined dynamic_lookup
else ifeq ($(UNAME_S),Linux)
  MODULE := graphlite_lua.so
  FFI_LIB := $(TARGET_RELEASE)/libgraphlite_ffi.so
  LDFLAGS_EXTRA := -Wl,-rpath,$(TARGET_RELEASE)
else
  MODULE := graphlite_lua.so
  FFI_LIB := $(TARGET_RELEASE)/libgraphlite_ffi.so
  LDFLAGS_EXTRA :=
endif

# Lua 5.4 - try pkg-config first, then common paths
LUA_CFLAGS := $(shell pkg-config --cflags lua5.4 2>/dev/null || pkg-config --cflags lua 2>/dev/null || echo "-I/usr/include/lua5.4 -I/usr/include")
LUA_LIBS := $(shell pkg-config --libs lua5.4 2>/dev/null || pkg-config --libs lua 2>/dev/null || echo "-llua5.4")

CC ?= gcc
CFLAGS := -std=c99 -Wall -O2 -fPIC $(LUA_CFLAGS) -I.
LDFLAGS := -shared -L$(TARGET_RELEASE) -lgraphlite_ffi $(LUA_LIBS) $(LDFLAGS_EXTRA)

.PHONY: all build run clean check-ffi

all: build

build: check-ffi $(MODULE)

check-ffi:
	@if [ ! -f "$(FFI_LIB)" ]; then \
		echo "Error: GraphLite FFI library not found at $(FFI_LIB)"; \
		echo "Build it first: cargo build --release -p graphlite-ffi"; \
		exit 1; \
	fi

$(MODULE): graphlite_lua.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)
	@echo "Built $@ successfully"

run: build
	LD_LIBRARY_PATH="$(TARGET_RELEASE):$$LD_LIBRARY_PATH" lua5.4 basic_usage.lua

run-mac: build
	DYLD_LIBRARY_PATH="$(TARGET_RELEASE):$$DYLD_LIBRARY_PATH" lua5.4 basic_usage.lua

clean:
	rm -f $(MODULE) graphlite_lua.so graphlite_lua.dylib graphlite_lua.dll

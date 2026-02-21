# Makefile for graphlite_lua -- Lua 5.4 C module for GraphLite
#
# Targets:
#   make          Build graphlite_lua.so
#   make run      Build and run basic_usage.lua
#   make clean    Remove build artefacts

UNAME := $(shell uname -s)

# Paths (relative to this directory)
FFI_LIB_DIR ?= ../../../target/release
LUA_PKG     ?= lua5.4

# Try pkg-config; fall back to common include paths
LUA_CFLAGS  := $(shell pkg-config --cflags $(LUA_PKG) 2>/dev/null \
                || pkg-config --cflags lua-5.4 2>/dev/null \
                || pkg-config --cflags lua54  2>/dev/null \
                || echo "-I/usr/include/lua5.4")

CC      ?= gcc
CFLAGS  := -std=c99 -O2 -Wall -Wextra -fPIC $(LUA_CFLAGS)
LDFLAGS := -L$(FFI_LIB_DIR) -lgraphlite_ffi

ifeq ($(UNAME),Darwin)
  # macOS: defer symbol resolution to load-time
  SO_EXT   := so
  LDFLAGS  += -undefined dynamic_lookup
  LIB_PATH  = DYLD_LIBRARY_PATH=$(FFI_LIB_DIR)
else
  SO_EXT   := so
  LIB_PATH  = LD_LIBRARY_PATH=$(FFI_LIB_DIR)
endif

TARGET := graphlite_lua.$(SO_EXT)

.PHONY: all run clean

all: $(TARGET)

$(TARGET): graphlite_lua.c
	$(CC) $(CFLAGS) -shared -o $@ $< $(LDFLAGS)

run: $(TARGET)
	$(LIB_PATH) lua5.4 basic_usage.lua

clean:
	rm -f $(TARGET)

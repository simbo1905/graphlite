SHELL := /bin/bash

CC ?= gcc

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
	MODULE_EXT := so
else ifeq ($(OS),Windows_NT)
	MODULE_EXT := dll
else
	MODULE_EXT := so
endif

MODULE_NAME := graphlite_lua
MODULE_FILE := $(MODULE_NAME).$(MODULE_EXT)

# Treat warnings as errors; allow unused callback parameters from FFI-style signatures.
STRICT_WARNINGS := -Wall -Wextra -Wpedantic -Werror -Wno-unused-parameter

CFLAGS ?= -O2 -std=c99 -fPIC
CPPFLAGS ?=
LDFLAGS ?= -shared
LDLIBS ?=

LUA_PKG ?= lua5.4
LUA_CFLAGS := $(shell pkg-config --cflags $(LUA_PKG) 2>/dev/null)
LUA_LIBS := $(shell pkg-config --libs $(LUA_PKG) 2>/dev/null)

FFI_INCLUDE := -I../../../graphlite-ffi
FFI_LIBDIR := -L../../../target/release
FFI_LIBS := -lgraphlite_ffi

.PHONY: all clean help check-deps

all: check-deps $(MODULE_FILE)

check-deps:
	@command -v pkg-config >/dev/null 2>&1 || { echo "ERROR: pkg-config not found"; exit 1; }
	@pkg-config --exists $(LUA_PKG) || { echo "ERROR: $(LUA_PKG) development package not found"; exit 1; }

$(MODULE_FILE): graphlite_lua.c
	$(CC) $(CFLAGS) $(STRICT_WARNINGS) $(CPPFLAGS) $(FFI_INCLUDE) $(LUA_CFLAGS) $< -o $@ $(LDFLAGS) $(FFI_LIBDIR) $(FFI_LIBS) $(LUA_LIBS) $(LDLIBS)

clean:
	rm -f graphlite_lua.so graphlite_lua.dylib graphlite_lua.dll graphlite_lua.bundle

help:
	@echo "Targets:"
	@echo "  make            Build $(MODULE_FILE) (warnings fail build)"
	@echo "  make clean      Remove built module artifacts"
	@echo "  make help       Show this help message"

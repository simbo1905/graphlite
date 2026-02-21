# GraphLite Lua 5.4 C Bindings Demo

A minimal Lua 5.4 "basic usage" demo using a tiny C99 shim module. This proves that **Lua 5.4 can embed and use GraphLite** via a custom C module.

**Note:** Lua 5.4 has no built-in FFI (unlike LuaJIT). This demo uses a tiny C module that links against the GraphLite Rust FFI shared library—comparable in spirit and scope to the Java BasicUsage demo.

## Prerequisites

- **Lua 5.4** (PUC Lua, not LuaJIT)
- **Lua 5.4 development files** (e.g. `liblua5.4-dev` on Debian/Ubuntu)
- **GraphLite FFI shared library** (built from this repo)

## Build Steps

### 1. Build the Rust FFI shared library

From the repository root:

```bash
cargo build --release -p graphlite-ffi
```

This produces:
- **Linux:** `target/release/libgraphlite_ffi.so`
- **macOS:** `target/release/libgraphlite_ffi.dylib`
- **Windows:** `target/release/graphlite_ffi.dll`

The `graphlite.h` header is generated in `graphlite-ffi/` during this build.

### 2. Build the Lua C module

From this directory (`examples/lua/bindings_c/`):

```bash
make
```

This compiles `graphlite_lua.c` into a shared library:
- **Linux:** `graphlite_lua.so`
- **macOS:** `graphlite_lua.dylib`

The Makefile uses:
- `-I.` for the GraphLite header (as it's bundled locally)
- `-L../../target/release -lgraphlite_ffi` for linking

### 3. Set runtime library path

The Lua module links against the GraphLite FFI library. At runtime, the dynamic linker must find it.

**Linux:**
```bash
export LD_LIBRARY_PATH="$(pwd)/../../target/release:$LD_LIBRARY_PATH"
```

**macOS:**
```bash
export DYLD_LIBRARY_PATH="$(pwd)/../../target/release:$DYLD_LIBRARY_PATH"
```

Or run via `make run`, which sets the path automatically.

## Run the demo

```bash
lua5.4 basic_usage.lua
```

Ensure `graphlite_lua.so` (or `.dylib`) is in the current directory or in `LUA_CPATH`, and that the GraphLite FFI library is in `LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH`.

## C Module API (Minimal)

| Function | Description |
|----------|-------------|
| `gl.version()` | Returns GraphLite version string |
| `gl.open(db_path)` | Opens database, returns db userdata |
| `db:create_session(user)` | Creates session, returns session_id string |
| `db:execute(session_id, query)` | Executes statement (DDL/INSERT), no return |
| `db:query(session_id, query)` | Executes query, returns `{rows = {...}, row_count = N}` |
| `db:close_session(session_id)` | Closes session |
| `db:close()` | Closes database |

**Error handling:** On FFI error, the module raises a Lua error with a clear message and the error code.

## FFI Mapping

| C Module | GraphLite FFI |
|----------|---------------|
| `gl.open` | `graphlite_open` |
| `db:close` | `graphlite_close` |
| `db:create_session` | `graphlite_create_session` |
| `db:execute` | `graphlite_query` (result discarded) |
| `db:query` | `graphlite_query` (JSON parsed to Lua tables) |
| `db:close_session` | `graphlite_close_session` |
| `gl.version` | `graphlite_version` |

Query results are returned as JSON from the FFI; the C module parses them into Lua tables (`rows` = array of row objects, each row = map of column name → value).

## Smoke Checklist

- [ ] Module loads: `require("graphlite_lua")` succeeds
- [ ] Version prints: `gl.version()` returns a string (e.g. "0.1.0")
- [ ] Queries return expected row counts (e.g. 3 persons, 2 over 25, aggregation row)
- [ ] Clean shutdown without crashes (session closed, db closed)

## Platform Notes

- **Linux:** Tested with Lua 5.4 and `liblua5.4-dev`. Use `make run`.
- **macOS:** Use `make run` to set `DYLD_LIBRARY_PATH`.
- **Windows:** Not tested. You would need to adapt the Makefile for `graphlite_lua.dll` and `graphlite_ffi.dll`, and ensure both are on `PATH` or in the same directory as the Lua script.

## Files

| File | Description |
|------|-------------|
| `graphlite_lua.c` | C99 Lua 5.4 module; FFI bindings + minimal JSON parser for results |
| `basic_usage.lua` | Demo script (mirrors Java BasicUsage flow) |
| `Makefile` | Builds the Lua module |
| `README.md` | This file |

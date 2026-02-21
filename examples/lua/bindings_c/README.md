# GraphLite Lua 5.4 Demo via Tiny C Module

This is a minimal "basic usage" demo for **PUC Lua 5.4** that uses a tiny C99 module (`graphlite_lua`) to call the existing GraphLite Rust FFI shared library.

> **Important:** Lua 5.4 has no built-in FFI; this demo uses a tiny C module.

This is intentionally small and low-level (similar in spirit to the Java `BasicUsage` example), not a full SDK.

## Files

- `graphlite_lua.c` - tiny Lua C module (`luaopen_graphlite_lua`)
- `Makefile` - builds module with strict warnings (`-Werror`)
- `setup.sh` - verifies Lua 5.4+/luarocks and installs `dkjson` locally
- `basic_usage.lua` - basic end-to-end demo script
- `README.md` - build/run instructions

## Minimal Lua API Exposed

- `gl.version() -> string`
- `gl.open(db_path) -> db`
- `db:create_session(user) -> session_id`
- `db:execute(session_id, query_string) -> nil`
- `db:query(session_id, query_string) -> json_bytes`
- `db:close_session(session_id)`
- `db:close()`

Errors are raised as Lua errors and include GraphLite error code + code name. JSON decoding is done in Lua with `dkjson`, not in C.

## Build

From repo root:

### 1) Build GraphLite Rust FFI shared library

```bash
cargo build --release -p graphlite-ffi
```

This produces:

- Linux: `target/release/libgraphlite_ffi.so`
- macOS: `target/release/libgraphlite_ffi.dylib`
- Windows: `target/release/graphlite_ffi.dll`

### 2) Enter demo directory

```bash
cd examples/lua/bindings_c
```

### 3) Install Lua JSON dependency (dkjson via luarocks)

Run setup (checks Lua exists and is >= 5.4, checks `luarocks`, installs `dkjson` into local `.rocks/`):

```bash
./setup.sh
```

`basic_usage.lua` automatically loads `dkjson` from this local `.rocks/` tree.

### 4) Build Lua C module

Preferred:

```bash
make
```

`Makefile` enforces strict warnings and fails the build on warnings (`-Werror`).  
For FFI-style callback signatures, `unused parameter` warnings are explicitly suppressed (`-Wno-unused-parameter`).

Manual compile examples (if needed):

#### Linux (gcc + pkg-config, lua5.4 headers/dev package required)

```bash
gcc -O2 -std=c99 -fPIC -Wall -Wextra -Wpedantic -Werror -Wno-unused-parameter \
  -shared graphlite_lua.c -o graphlite_lua.so \
  -I../../../graphlite-ffi \
  $(pkg-config --cflags lua5.4) \
  -L../../../target/release -lgraphlite_ffi \
  $(pkg-config --libs lua5.4)
```

#### macOS (clang + pkg-config)

```bash
clang -O2 -std=c99 -fPIC -Wall -Wextra -Wpedantic -Werror -Wno-unused-parameter \
  -shared graphlite_lua.c -o graphlite_lua.so \
  -I../../../graphlite-ffi \
  $(pkg-config --cflags lua5.4) \
  -L../../../target/release -lgraphlite_ffi \
  $(pkg-config --libs lua5.4)
```

(`graphlite_lua.dylib` also works; update `package.cpath` if needed.)

#### Windows (MinGW example)

```bash
gcc -O2 -std=c99 -Wall -Wextra -Wpedantic -Werror -Wno-unused-parameter \
  -shared graphlite_lua.c -o graphlite_lua.dll \
  -I../../../graphlite-ffi \
  -IC:/lua/include -LC:/lua/lib -llua54 \
  -L../../../target/release -lgraphlite_ffi
```

Adjust Lua include/lib paths for your installation.

## Runtime library path

The Lua module links against GraphLite FFI, so the OS loader must find that shared library at runtime.

### Linux

```bash
export LD_LIBRARY_PATH="$(pwd)/../../../target/release:${LD_LIBRARY_PATH}"
```

### macOS

```bash
export DYLD_LIBRARY_PATH="$(pwd)/../../../target/release:${DYLD_LIBRARY_PATH}"
```

### Windows (PowerShell)

```powershell
$env:PATH = "$(Resolve-Path ..\..\..\target\release);$env:PATH"
```

## Run

From `examples/lua/bindings_c`:

```bash
lua5.4 basic_usage.lua
```

The demo loads the module with:

```lua
local gl = require("graphlite_lua")
```

And decodes query JSON bytes with:

```lua
local json = require("dkjson")
```

## Git ignore for downloaded parser

`setup.sh` installs `dkjson` under `.rocks/` and `.rocks/` is ignored by `examples/lua/bindings_c/.gitignore`.

## Smoke Checklist

- `require("graphlite_lua")` succeeds
- `require("dkjson")` succeeds after running `./setup.sh`
- `gl.version()` prints a version string
- `db:query(...)` returns JSON bytes and `dkjson.decode(...)` succeeds
- basic query returns `row_count == 3` for all persons
- filtered query returns `row_count == 2` for `age > 25`
- aggregation query returns one row (count + avg)
- session/database close cleanly without crash

## FFI Mapping Used by the C Module

`graphlite_lua.c` directly uses:

- `graphlite_version`
- `graphlite_open` / `graphlite_close`
- `graphlite_create_session` / `graphlite_close_session`
- `graphlite_query`
- `graphlite_free_string`

`db:query` returns the raw JSON payload from `graphlite_query` (as a Lua string/bytes).
`db:execute` is a thin wrapper over `graphlite_query` that ignores the returned JSON payload.

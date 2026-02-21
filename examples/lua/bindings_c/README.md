# GraphLite Lua 5.4 Basic Usage Demo

Lua 5.4 has no built-in FFI; this demo uses a **tiny C module**
(`graphlite_lua.c`) that calls the same GraphLite FFI shared library used by
the Python and Java bindings.

This is *not* a full SDK or binding generator — just enough to prove that
Lua 5.4 can embed/use GraphLite via a custom C module.

## Files

| File | Description |
|------|-------------|
| `graphlite_lua.c` | Minimal C99 Lua module (~300 lines). Links against the Rust FFI shared library and exposes a small Lua API. Includes a tiny JSON-to-Lua-table parser for query results. |
| `basic_usage.lua` | Demo script mirroring the Java `BasicUsage.java`. Opens a database, creates a schema/graph, inserts persons, queries, and prints results. |
| `Makefile` | Builds the shared module on Linux and macOS. |

## Prerequisites

- **Rust toolchain** (to build the GraphLite FFI shared library)
- **Lua 5.4** development headers and interpreter
  - Debian/Ubuntu: `sudo apt install lua5.4 liblua5.4-dev`
  - macOS (Homebrew): `brew install lua@5.4`
  - Fedora: `sudo dnf install lua-devel`
- **GCC** or **Clang** (C99)

## Build Steps

### 1. Build the Rust FFI shared library

From the repository root:

```bash
cargo build --release -p graphlite-ffi
```

This produces `target/release/libgraphlite_ffi.so` (Linux),
`target/release/libgraphlite_ffi.dylib` (macOS), or
`target/release/graphlite_ffi.dll` (Windows).

### 2. Build the Lua C module

From this directory (`examples/lua/bindings_c/`):

```bash
make
```

Or manually:

**Linux:**
```bash
gcc -std=c99 -shared -fPIC -o graphlite_lua.so graphlite_lua.c \
    $(pkg-config --cflags lua5.4) \
    -L../../../target/release -lgraphlite_ffi
```

**macOS:**
```bash
gcc -std=c99 -shared -fPIC -undefined dynamic_lookup \
    -o graphlite_lua.so graphlite_lua.c \
    $(pkg-config --cflags lua5.4) \
    -L../../../target/release -lgraphlite_ffi
```

### 3. Run the demo

Set the shared library search path so the Rust FFI library can be found at
runtime, then run:

**Linux:**
```bash
LD_LIBRARY_PATH=../../../target/release lua5.4 basic_usage.lua
```

**macOS:**
```bash
DYLD_LIBRARY_PATH=../../../target/release lua5.4 basic_usage.lua
```

Or use the Makefile shortcut:

```bash
make run
```

## C Module API

The module exposes just enough for the demo:

| Lua call | FFI function called | Returns |
|----------|-------------------|---------|
| `gl.version()` | `graphlite_version` | version string |
| `gl.open(path)` | `graphlite_open` | db userdata |
| `db:create_session(user)` | `graphlite_create_session` | session ID string |
| `db:execute(sid, query)` | `graphlite_query` (result discarded) | nil |
| `db:query(sid, query)` | `graphlite_query` → JSON → Lua table | `{variables={...}, rows={...}, row_count=N}` |
| `db:close_session(sid)` | `graphlite_close_session` | nil |
| `db:close()` | `graphlite_close` | nil |

All FFI-allocated strings are freed with `graphlite_free_string`.
The db userdata has a `__gc` metamethod that calls `graphlite_close`
automatically if the user forgets `db:close()`.

### Error handling

If any FFI call returns an error, a Lua error is raised with a message
containing the error code name, numeric code, and context string.

## Expected Output

```
=== GraphLite Lua 5.4 Bindings Example ===

Using temporary database: /tmp/lua_XXXXXX_graphlite

1. Opening database...
   [OK] GraphLite version: 0.1.0

2. Creating session...
   [OK] Session created: xxxxxxxx-xxxx-xxxx...

3. Setting up schema and graph...
   [OK] Schema and graph created

4. Inserting data...
   [OK] Inserted 3 persons

5. Querying: All persons (name, age)
   Found 3 persons:
   - Alice: 30 years old
   - Bob: 25 years old
   - Charlie: 35 years old

6. Filtering: Persons older than 25 (ascending by age)
   Found 2 persons over 25:
   - Alice: 30 years old
   - Charlie: 35 years old

7. Aggregation query...
   Total persons: 3
   Average age: 30.0

8. Closing session...
   [OK] Session closed

9. Closing database...
   [OK] Database closed

=== Example completed successfully ===
```

## Smoke Checklist

- [ ] `require("graphlite_lua")` loads without error
- [ ] `gl.version()` returns a version string
- [ ] Step 5 query returns 3 rows
- [ ] Step 6 query returns 2 rows (age > 25)
- [ ] Step 7 aggregation returns count 3
- [ ] Clean shutdown (no crash, no leak)

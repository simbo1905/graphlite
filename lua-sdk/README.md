# GraphLite LuaJIT High-Level SDK

A session-centric, high-level Lua API for GraphLite that mirrors the Python SDK semantics. Uses LuaJIT's built-in `ffi` library to bind against the GraphLite C FFI shared library.

**This SDK is LuaJIT-only (Lua 5.1 compatible), not PUC Lua 5.4.**

## Architecture

```
Your Application
      ↓
GraphLite SDK (lua-sdk/src/)
      ↓
GraphLite FFI Adapter (graphlite_ffi.lua)
      ↓
libgraphlite_ffi.so / .dylib / .dll (Rust)
```

## Prerequisites

1. **Build the GraphLite FFI library**:
   ```bash
   cd /path/to/GraphLite
   cargo build --release -p graphlite-ffi
   ```

2. **LuaJIT** (2.0 or 2.1):
   ```bash
   # Ubuntu/Debian
   sudo apt install luajit

   # macOS
   brew install luajit
   ```

3. **dkjson** (required for JSON parsing; engine returns JSON as bytes):
   ```bash
   luarocks install dkjson
   ```
   Or run `./setup.sh` from examples/lua/sdk/.

## API

### Connection

```lua
local connection = require("src.connection")
local GraphLite = connection.GraphLite

local db = GraphLite.open("./mydb")
-- ...
db:close()
```

### Session

```lua
local session = db:session("admin")
session:execute("CREATE SCHEMA IF NOT EXISTS /example")
session:execute("SESSION SET SCHEMA /example")
session:execute("CREATE GRAPH IF NOT EXISTS social")
session:execute("SESSION SET GRAPH social")
session:execute("INSERT (p:Person {name: 'Alice', age: 30})")

local result = session:query("MATCH (p:Person) RETURN p.name, p.age")
for _, row in ipairs(result.rows) do
  print(row["p.name"], row["p.age"])
end

session:close()
db:close()
```

### Error Handling

```lua
local errors = require("src.errors")
local ok, err = pcall(function()
  local db = GraphLite.open("/nonexistent")
end)
if not ok then
  -- err is a table with .message, .code, .code_name
  print(err.message)
end
```

## Module Layout

```
lua-sdk/
├── src/
│   ├── graphlite_ffi.lua   -- FFI bindings (cdef + load)
│   ├── connection.lua      -- GraphLite.open(), db:session(), db:close()
│   ├── session.lua         -- session:query(), session:execute(), session:close()
│   ├── errors.lua          -- Typed error tables
│   ├── result.lua          -- QueryResult with flattened rows
│   └── json_util.lua       -- JSON parsing (dkjson, no embedded parser)
├── tests/
│   └── smoke_test.lua      -- Minimal smoke test
└── README.md
```

## Usage from Examples

The examples live on `main` under `examples/lua/sdk/`. They auto-locate this SDK at:

- `~/github/simbo1905/graphlite/lua-sdk/`
- `~/github/deepgraphai/GraphLite/lua-sdk/`

Or set `GRAPH_LITE_LUA_SDK` to your SDK path.

```bash
cd examples/lua/sdk
luajit drug_discovery.lua
```

## Branching

- **`luajit-sdk` branch**: Contains this `lua-sdk/` implementation.
- **`main` branch**: Contains only `examples/lua/sdk/` (examples + path bootstrapper).

A fresh clone of `main` does not include the full SDK; the example instructs users to checkout the `luajit-sdk` branch (or clone it) to get `lua-sdk/`.

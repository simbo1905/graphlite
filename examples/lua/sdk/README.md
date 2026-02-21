# GraphLite LuaJIT High-Level SDK Examples

This directory contains examples using the GraphLite High-Level LuaJIT SDK (from the `luajit-sdk` branch).

## Overview

The High-Level SDK provides an ergonomic, session-centric API for GraphLite with:
- Session-centric API (session objects instead of session IDs)
- Typed error tables (ConnectionError, SessionError, QueryError)
- Cleaner interface mirroring the Python SDK
- LuaJIT FFI bindings to the same GraphLite C library

## Architecture

```
Your Application
      ↓
GraphLite SDK (lua-sdk/src/)
      ↓
GraphLite FFI Adapter (graphlite_ffi.lua)
      ↓
libgraphlite_ffi.so (Rust)
```

## Setup

### Prerequisites

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

3. **LuaJIT SDK Dependency**

   The high-level LuaJIT SDK is on the `luajit-sdk` branch:

   ```bash
   # Clone and checkout luajit-sdk branch
   cd ~/github/simbo1905  # or ~/github/deepgraphai
   git clone https://github.com/simbo1905/graphlite.git  # if not already cloned
   cd graphlite
   git fetch origin luajit-sdk
   git checkout luajit-sdk
   ```

   The examples will automatically find the SDK at:
   - `~/github/simbo1905/graphlite/lua-sdk/`
   - `~/github/deepgraphai/GraphLite/lua-sdk/`

   Or set `GRAPH_LITE_LUA_SDK` to your lua-sdk path.

## Examples

### Drug Discovery Example

A pharmaceutical research example demonstrating:
- Modeling proteins (targets), compounds (drugs), and assays
- Relationships: TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS
- IC50 filtering, pathway traversal, aggregation

**Run:**
```bash
cd examples/lua/sdk
luajit drug_discovery.lua
```

### Basic Usage

Minimal sanity test: open DB, create session, insert nodes, query, close.

**Run:**
```bash
luajit basic_usage.lua
```

## API (mirrors Python SDK)

### Low-Level FFI Bindings
```lua
-- (conceptual - we use the high-level SDK)
db = GraphLite("./mydb")
session_id = db.create_session("admin")
result = db.query(session_id, "MATCH (n) RETURN n")
```

### High-Level SDK (This Example)
```lua
local GraphLite = require("src.connection").GraphLite

db = GraphLite.open("./mydb")
session = db:session("admin")
result = session:query("MATCH (n) RETURN n")
session:close()
db:close()
```

**Key Differences**:
1. Use `.open()` instead of constructor
2. Session object with methods instead of session ID
3. Cleaner, session-centric API
4. Typed error tables (ConnectionError, SessionError, QueryError)

## Requirements

- LuaJIT 2.0+ (Lua 5.1 semantics)
- GraphLite FFI library (built from GraphLite repository)
- **This SDK is LuaJIT-only, not PUC Lua 5.4**

## Platform Notes

- **Linux/macOS**: Fully supported
- **Windows**: Path handling may differ; set `GRAPH_LITE_LUA_SDK` explicitly if needed

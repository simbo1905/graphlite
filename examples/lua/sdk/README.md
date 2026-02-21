# GraphLite LuaJIT High-Level SDK Examples

This directory contains examples using the GraphLite High-Level LuaJIT SDK (from the `luajit-sdk` branch).

> **This SDK is LuaJIT-only (Lua 5.1 compatible via LuaJIT), not PUC Lua 5.4.**

## Overview

The High-Level SDK provides an ergonomic, session-centric API for GraphLite with:
- Session-centric API (session objects instead of session IDs)
- Typed error hierarchy (Lua tables with metatables)
- Cleaner interface matching the Rust and Python SDKs
- Automatic resource cleanup via `__gc` best-effort finalizers

## Architecture

```
Your Application (drug_discovery.lua)
      ↓
GraphLite LuaJIT SDK (lua-sdk/src/)
      ↓
graphlite_ffi.lua  (ffi.cdef + ffi.load wrapper)
      ↓
libgraphlite_ffi.so / .dylib / .dll  (Rust FFI)
```

## Setup

### Prerequisites

1. **LuaJIT** (2.0+ or 2.1-beta):
   ```bash
   # Ubuntu / Debian
   sudo apt-get install luajit

   # macOS (Homebrew)
   brew install luajit
   ```

2. **Build the GraphLite FFI library**:
   ```bash
   cd ~/github/simbo1905/graphlite
   cargo build --release -p graphlite-ffi
   ```

3. **LuaJIT SDK Dependency**

   The LuaJIT SDK is on the `luajit-sdk` branch of this repository:

   ```bash
   cd ~/github/simbo1905/graphlite
   git fetch origin luajit-sdk
   git worktree add ../graphlite-luajit-sdk luajit-sdk
   # Or simply checkout the branch in a separate clone:
   # git clone https://github.com/simbo1905/graphlite.git graphlite-luajit-sdk
   # cd graphlite-luajit-sdk && git checkout luajit-sdk
   ```

   The examples will automatically find the SDK using this search order:

   1. `GRAPHLITE_LUA_SDK` environment variable (if set)
   2. `~/github/simbo1905/graphlite/lua-sdk/`

   If neither is found, the script prints a clear error with setup instructions.

## Examples

### Drug Discovery Example

A comprehensive pharmaceutical research example demonstrating:
- Modeling proteins (disease targets), compounds (drugs), and assays (experiments)
- Creating relationships: TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS
- Real-world data: IC50 measurements, clinical trial phases
- Analytical queries: IC50 filtering, pathway traversal, aggregation

**Run:**
```bash
cd examples/lua/sdk
LD_LIBRARY_PATH=../../../target/release luajit drug_discovery.lua
```

### Basic Usage Example

A minimal example covering open/session/insert/query/close:

```bash
cd examples/lua/sdk
LD_LIBRARY_PATH=../../../target/release luajit basic_usage.lua
```

## API Differences from Low-Level FFI

### Low-Level FFI (direct ffi.load)
```lua
local ffi = require("ffi")
local lib = ffi.load("graphlite_ffi")
-- manually manage C pointers, error codes, string freeing…
```

### High-Level SDK (This Example)
```lua
local GraphLite = require("src.connection").GraphLite
local db = GraphLite.open("./mydb")
local session = db:session("admin")
local result = session:query("MATCH (n) RETURN n")
session:close()
db:close()
```

**Key Differences**:
1. Use `.open()` static method instead of raw `ffi.load`
2. Session object with methods instead of manual session ID strings
3. Cleaner, session-centric API
4. Typed errors (ConnectionError, SessionError, QueryError, etc.)
5. Automatic JSON decoding into Lua tables
6. Best-effort `__gc` finalizers for resource cleanup

## Requirements

- LuaJIT 2.0+ (Lua 5.1 semantics)
- GraphLite FFI library (built from this repository)
- The `luajit-sdk` branch checked out for the SDK source

## Domain Model (Drug Discovery)

```
Compound → TESTED_IN → Assay → MEASURES_ACTIVITY_ON → Target (Protein)
Compound → INHIBITS → Target (with IC50 measurements)
```

**Use Cases**: Target-based drug discovery, compound optimization, clinical trial tracking, pharmaceutical knowledge graphs.

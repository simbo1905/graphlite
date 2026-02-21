# GraphLite Lua High-Level SDK Examples

This directory contains examples that use the GraphLite **Lua high-level SDK** from the `luajit-sdk` branch.

> **Requires Lua 5.4+** and `luarocks` for dependency management.
>
> The full SDK implementation is intentionally **not** stored on `main`.
> These examples auto-locate a separate `lua-sdk/` checkout.

## Overview

The LuaJIT SDK provides an ergonomic, session-centric API similar to the Python SDK pattern:

- `GraphLite.open(path)` returns a connection object
- `db:session(user)` returns a session object (no raw session ID in normal use)
- `session:execute(...)` for statements
- `session:query(...)` for result rows
- typed Lua error objects

## Architecture

```text
Your LuaJIT app (this directory)
      |
      v
examples/lua/sdk/sdk_locator.lua
      |
      v
lua-sdk/src/ (from luajit-sdk branch checkout)
  - connection.lua
  - session.lua
  - errors.lua
  - graphlite_ffi.lua
      |
      v
libgraphlite_ffi.{so,dylib,dll} (Rust FFI)
```

## Setup

### Prerequisites

1. **Lua 5.4+** and **luarocks**:
   ```bash
   # Ubuntu / Debian
   sudo apt-get install lua5.4 luarocks

   # macOS (Homebrew)
   brew install lua luarocks
   ```

2. **Build the GraphLite FFI library**:
   ```bash
   cargo build --release -p graphlite-ffi
   ```

3. **Checkout the Lua SDK branch** and run setup:
   ```bash
   cd ~/github/deepgraphai
   git clone https://github.com/deepgraphai/GraphLite.git  # if needed
   cd GraphLite
   git checkout luajit-sdk
   cd lua-sdk
   ./setup.sh   # checks Lua >= 5.4, luarocks, installs dkjson
   ```

### SDK Auto-Discovery

The examples resolve SDK path in this order:

1. `GRAPH_LITE_LUA_SDK` environment variable
2. default path: `~/github/deepgraphai/GraphLite/lua-sdk`

If neither works, the script prints clear setup instructions.

## Examples

### Drug Discovery Example

A representative pharmaceutical graph workflow with:

- proteins/targets, compounds, assays
- relationships (`TESTED_IN`, `MEASURES_ACTIVITY_ON`, `INHIBITS`)
- analytics:
  - IC50 filtering
  - traversal query
  - aggregation query

Run:

```bash
cd examples/lua/sdk
lua drug_discovery.lua
```

### Basic Usage (Smoke Test)

Creates a temporary database, inserts a few nodes, queries rows, and cleans up.

Run:

```bash
cd examples/lua/sdk
lua basic_usage.lua
```

## Important Notes

- Requires **Lua 5.4+** (not LuaJIT or PUC Lua 5.1/5.2/5.3).
- JSON parsing uses `dkjson` installed via `luarocks` (no embedded JSON code).
- On Windows, set `GRAPH_LITE_LUA_SDK` and ensure `graphlite_ffi.dll` is discoverable.

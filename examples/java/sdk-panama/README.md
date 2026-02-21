# GraphLite Java SDK (Panama FFM) Examples

Examples demonstrating the GraphLite High-Level Java SDK built on Java's
**Foreign Function & Memory API** (Project Panama) — no JNI required.

## Overview

These examples use the session-centric Java SDK that mirrors the Python SDK:

| Python SDK | Java SDK (Panama) |
|---|---|
| `db = GraphLite.open("./mydb")` | `var db = GraphLite.open("./mydb")` |
| `session = db.session("admin")` | `var session = db.session("admin")` |
| `result = session.query(q)` | `var result = session.query(q)` |
| `for row in result.rows:` | `for (var row : result.rows())` |
| Context managers (`with`) | try-with-resources |
| Typed exceptions | Typed exceptions (`ConnectionException`, etc.) |

## Prerequisites

### 1. Build the Rust FFI library

```bash
cd <graphlite-repo-root>
cargo build --release -p graphlite-ffi
```

### 2. Set the library path

**Linux:**
```bash
export LD_LIBRARY_PATH=$(pwd)/target/release
```

**macOS:**
```bash
export DYLD_LIBRARY_PATH=$(pwd)/target/release
```

Or point directly:
```bash
export GRAPHLITE_FFI_LIB=$(pwd)/target/release/libgraphlite_ffi.so   # Linux
export GRAPHLITE_FFI_LIB=$(pwd)/target/release/libgraphlite_ffi.dylib # macOS
```

### 3. Build and install the Java SDK

```bash
cd java-sdk
mvn install -DskipTests
```

### 4. Build the examples

```bash
cd examples/java/sdk-panama
mvn compile
```

## Running the Examples

### Drug Discovery Demo

Full-featured pharmaceutical research example: proteins, compounds, assays,
IC50 measurements, graph traversals, and aggregation queries.

```bash
mvn exec:java -Dexec.mainClass="DrugDiscovery" \
    -Dgraphlite.ffi.lib=../../../target/release/libgraphlite_ffi.so
```

### Basic Usage

Simple introduction to the SDK: open, session, insert, query, close.

```bash
mvn exec:java -Dexec.mainClass="BasicUsage" -Pbasic \
    -Dgraphlite.ffi.lib=../../../target/release/libgraphlite_ffi.so
```

> **Note:** Add `--enable-native-access=ALL-UNNAMED` via `MAVEN_OPTS` if
> your JDK requires it:
> ```bash
> export MAVEN_OPTS="--enable-native-access=ALL-UNNAMED"
> ```

## Complete From-Scratch Sequence

```bash
# 1. Clone and build the FFI library
git clone https://github.com/simbo1905/graphlite.git
cd graphlite
cargo build --release -p graphlite-ffi

# 2. Set library path
export LD_LIBRARY_PATH=$(pwd)/target/release        # Linux
# export DYLD_LIBRARY_PATH=$(pwd)/target/release    # macOS

# 3. Build and install the Java SDK
cd java-sdk
mvn install -DskipTests

# 4. Build and run the examples
cd ../examples/java/sdk-panama
export MAVEN_OPTS="--enable-native-access=ALL-UNNAMED"
mvn compile exec:java -Dexec.mainClass="DrugDiscovery" \
    -Dgraphlite.ffi.lib=$(pwd)/../../../target/release/libgraphlite_ffi.so
```

## Domain Model (Drug Discovery)

```
Compound -> TESTED_IN -> Assay -> MEASURES_ACTIVITY_ON -> Target (Protein)
Compound -> INHIBITS -> Target (with IC50 measurements)
```

The demo creates 4 proteins, 4 compounds, 4 assays, and links them with
TESTED_IN, MEASURES_ACTIVITY_ON, and INHIBITS relationships carrying real
pharmaceutical data (IC50, Ki, selectivity index).

## Requirements

| Requirement | Version |
|---|---|
| Java (JDK) | 22+ (targets Panama FFM API, JEP 454) |
| Maven | 3.9+ |
| Rust / Cargo | stable |

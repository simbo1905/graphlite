# GraphLite Java SDK (Panama) Examples

Examples using the GraphLite high-level Java SDK with the Foreign Function & Memory API.

## Prerequisites

- Java 21+ (22+ recommended)
- Maven 3.9+
- GraphLite FFI library (built from GraphLite repo)

## Setup (from scratch)

### 1. Build the Rust FFI library

```bash
cd /path/to/graphlite
cargo build --release -p graphlite-ffi
```

### 2. Set library path (Linux/macOS)

```bash
# Linux
export LD_LIBRARY_PATH=/path/to/graphlite/target/release:$LD_LIBRARY_PATH

# macOS
export DYLD_LIBRARY_PATH=/path/to/graphlite/target/release:$DYLD_LIBRARY_PATH
```

Or set the full path:
```bash
export GRAPHLITE_FFI_LIB=/path/to/graphlite/target/release/libgraphlite_ffi.so   # Linux
export GRAPHLITE_FFI_LIB=/path/to/graphlite/target/release/libgraphlite_ffi.dylib # macOS
```

### 3. Install the Java SDK

```bash
cd /path/to/graphlite/java-sdk-panama
mvn clean install
```

### 4. Run the examples

```bash
cd /path/to/graphlite/examples/java/sdk-panama

# Drug Discovery demo (feature-parity with Python SDK)
mvn exec:java -Dexec.mainClass="io.graphlite.examples.DrugDiscovery"

# Basic usage
mvn exec:java -Dexec.mainClass="io.graphlite.examples.BasicUsage"
```

## Examples

- **DrugDiscovery** – Pharmaceutical research: targets, compounds, assays, relationships (TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS), analytical queries
- **BasicUsage** – Simple schema setup, insert, query, filter, aggregation

## Windows

The SDK supports Windows; use `graphlite_ffi.dll`. If you encounter issues, set `GRAPHLITE_FFI_LIB` to the full path of the DLL.

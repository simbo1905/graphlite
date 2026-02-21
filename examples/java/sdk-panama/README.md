# GraphLite Java SDK (Panama) Examples

This directory contains high-level Java SDK examples that mirror the Python SDK demo style, while using Java 25 FFM/Panama bindings under the hood.

## Included Examples

- `DrugDiscovery` - feature-parity demo with the Python SDK drug-discovery workflow
- `BasicUsage` - smaller CRUD/session example

## From-Scratch Setup

### 1) Build the Rust FFI library

```bash
cd /path/to/graphlite
cargo build --release -p graphlite-ffi
```

### 2) Set native library resolution (recommended)

#### Linux
```bash
export GRAPHLITE_FFI_LIB="/path/to/graphlite/target/release/libgraphlite_ffi.so"
export LD_LIBRARY_PATH="/path/to/graphlite/target/release:${LD_LIBRARY_PATH}"
```

#### macOS
```bash
export GRAPHLITE_FFI_LIB="/path/to/graphlite/target/release/libgraphlite_ffi.dylib"
export DYLD_LIBRARY_PATH="/path/to/graphlite/target/release:${DYLD_LIBRARY_PATH}"
```

### 3) Build/install the Java SDK locally

The examples intentionally depend on the Java SDK artifact (to keep examples lean).

If your current branch does not contain `java-sdk/`, check out the branch that hosts the SDK module (for example `java-panama-sdk`) and install from there.

```bash
cd /path/to/graphlite/java-sdk
mvn clean install
```

### 4) Run an example

```bash
cd /path/to/graphlite/examples/java/sdk-panama

# Drug discovery demo
mvn -q compile exec:java -Dexec.mainClass=DrugDiscovery

# Optional basic usage demo
mvn -q compile exec:java -Dexec.mainClass=BasicUsage
```

## Notes

- Windows is not supported by this Panama demo path yet; use Linux/macOS (or WSL).
- If `GRAPHLITE_FFI_LIB` is set, it overrides all default lookup behavior.

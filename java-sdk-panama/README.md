# GraphLite Java SDK (Panama/FFM)

High-level Java SDK for GraphLite using the Foreign Function & Memory API (Project Panama).
Targets Java 21+ (FFM API: preview in 21, stable in 22+). No JNI—Panama-first.

## Features

- **Session-centric API**: `GraphLite.open(path)` → `db.session(user)` → `session.query(...)`
- **Typed exceptions**: `ConnectionException`, `SessionException`, `QueryException` with error codes
- **AutoCloseable**: `GraphLite` and `Session` implement `AutoCloseable`
- **QueryResult**: `first()`, `isEmpty()`, `column()`, row iteration

## Setup

### 1. Build the Rust FFI library

```bash
cd /path/to/graphlite
cargo build --release -p graphlite-ffi
```

The shared library will be at `target/release/libgraphlite_ffi.so` (Linux), `libgraphlite_ffi.dylib` (macOS), or `graphlite_ffi.dll` (Windows).

### 2. Library path

The SDK searches for the library in:
- `target/release`, `target/debug` (relative to cwd and parent dirs)
- `/usr/local/lib`, `/usr/lib`

Override with environment variable:
```bash
export GRAPHLITE_FFI_LIB=/path/to/libgraphlite_ffi.so
```

On Linux/macOS, you may need:
```bash
export LD_LIBRARY_PATH=/path/to/graphlite/target/release:$LD_LIBRARY_PATH   # Linux
export DYLD_LIBRARY_PATH=/path/to/graphlite/target/release:$DYLD_LIBRARY_PATH  # macOS
```

### 3. Build the Java SDK

```bash
cd java-sdk-panama
mvn clean install
```

## Usage

```java
import io.graphlite.sdk.GraphLite;
import io.graphlite.sdk.Session;
import io.graphlite.sdk.QueryResult;

try (GraphLite db = GraphLite.open("./mydb")) {
    try (Session session = db.session("admin")) {
        session.execute("CREATE SCHEMA IF NOT EXISTS /example");
        session.execute("SESSION SET SCHEMA /example");
        session.execute("CREATE GRAPH IF NOT EXISTS social");
        session.execute("SESSION SET GRAPH social");
        session.execute("INSERT (:Person {name: 'Alice', age: 30})");

        QueryResult result = session.query("MATCH (p:Person) RETURN p.name, p.age");
        for (Map<String, Object> row : result.getRows()) {
            System.out.println(row);
        }
        if (!result.isEmpty()) {
            Map<String, Object> first = result.first().orElseThrow();
            List<Object> names = result.column("p.name");
        }
    }
}
```

## Architecture

- **io.graphlite.sdk.GraphLite** – High-level entry point
- **io.graphlite.sdk.Session** – Session management, query/execute
- **io.graphlite.sdk.QueryResult** – Result iteration, first(), column()
- **io.graphlite.sdk.Errors** – Typed exception hierarchy
- **io.graphlite.sdk.ffi.GraphLiteFFI** – All FFM bindings (FFI boundary)
- **io.graphlite.sdk.ffi.Layouts** – MemoryLayout definitions

## Requirements

- Java 21+ (22+ recommended for stable FFM)
- GraphLite FFI library (built from GraphLite repo)

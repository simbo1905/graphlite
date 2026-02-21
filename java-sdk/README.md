# GraphLite Java SDK (Panama FFM)

A high-level Java SDK for [GraphLite](https://github.com/simbo1905/graphlite) that uses
Java's **Foreign Function & Memory API** (Project Panama, [JEP 454](https://openjdk.org/jeps/454))
to call the native `libgraphlite_ffi` shared library — no JNI required.

## Architecture

```
Your Application
       |
       v
+--------------------------------------+
|  GraphLite SDK (this module)         |
|  - GraphLite   (AutoCloseable)       |
|  - Session     (AutoCloseable)       |
|  - QueryResult (immutable rows)      |
|  - Errors      (exception hierarchy) |
+--------------------------------------+
       |  Panama FFM downcalls
       v
+--------------------------------------+
|  ffi/GraphLiteFFI.java               |
|  (Linker + SymbolLookup + Arena)     |
+--------------------------------------+
       |
       v
+--------------------------------------+
|  libgraphlite_ffi.so / .dylib        |
|  (Rust FFI, built with Cargo)        |
+--------------------------------------+
```

## Requirements

| Requirement | Version |
|---|---|
| Java (JDK) | 22+ (tested with 25) |
| Maven | 3.9+ |
| Rust / Cargo | stable (for building the FFI library) |

The FFM API (JEP 454) was finalized in Java 22. This SDK targets Java 22+ and
works out of the box with Java 25.

## Quick Start

### 1. Build the native FFI library

```bash
cd <graphlite-repo-root>
cargo build --release -p graphlite-ffi
```

This produces:
- **Linux:** `target/release/libgraphlite_ffi.so`
- **macOS:** `target/release/libgraphlite_ffi.dylib`

### 2. Build and install the Java SDK

```bash
cd java-sdk
mvn install -DskipTests
```

### 3. Use it in your project

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>io.graphlite</groupId>
    <artifactId>graphlite-sdk-panama</artifactId>
    <version>0.1.0</version>
</dependency>
```

### 4. Run your code

```bash
# Tell the JVM where the native library lives
export LD_LIBRARY_PATH=/path/to/graphlite/target/release   # Linux
export DYLD_LIBRARY_PATH=/path/to/graphlite/target/release  # macOS

java --enable-native-access=ALL-UNNAMED -cp ... YourApp
```

Or set the library path explicitly:

```bash
java --enable-native-access=ALL-UNNAMED \
     -Dgraphlite.ffi.lib=/path/to/libgraphlite_ffi.so \
     -cp ... YourApp
```

Or use the `GRAPHLITE_FFI_LIB` environment variable:

```bash
export GRAPHLITE_FFI_LIB=/path/to/libgraphlite_ffi.so
java --enable-native-access=ALL-UNNAMED -cp ... YourApp
```

## API Overview

```java
try (var db = GraphLite.open("/tmp/mydb")) {
    try (var session = db.session("admin")) {
        session.execute("CREATE GRAPH IF NOT EXISTS g");
        session.execute("SESSION SET GRAPH g");

        session.execute("INSERT (:Person {name: 'Alice', age: 30})");

        QueryResult result = session.query(
            "MATCH (p:Person) RETURN p.name AS name, p.age AS age"
        );

        for (var row : result.rows()) {
            System.out.println(row.get("name") + ": " + row.get("age"));
        }
    }
}
```

### Key Classes

| Class | Description |
|---|---|
| `GraphLite` | Database connection entry point. `GraphLite.open(path)` returns an `AutoCloseable` handle. |
| `Session` | User session. `db.session(user)` returns an `AutoCloseable` session. |
| `QueryResult` | Immutable result of a query. Provides `rows()`, `first()`, `isEmpty()`, `column(name)`, `variables()`. |
| `Errors` | Exception hierarchy: `ConnectionException`, `SessionException`, `QueryException`, `LibraryLoadException`, `SerializationException`. |
| `ffi/GraphLiteFFI` | All Panama FFM bindings centralized. Linker + SymbolLookup + downcall MethodHandles. |
| `ffi/Layouts` | `MemoryLayout` definitions for FFI struct types. |

### Library Resolution

The native library is located in this order:

1. `GRAPHLITE_FFI_LIB` environment variable (full path)
2. `graphlite.ffi.lib` system property (full path)
3. Relative paths: `target/release/`, `../target/release/`, etc.
4. System library paths (`LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` / `PATH`)

### Exception Hierarchy

```
GraphLiteException (base, carries ErrorCode)
  +-- ConnectionException  (database open failures)
  +-- SessionException     (session create/close failures)
  +-- QueryException       (query execution failures)
  +-- LibraryLoadException (native library not found)
  +-- SerializationException (JSON parse failures)
```

## Running the Smoke Test

```bash
# Make sure the FFI library is built
cargo build --release -p graphlite-ffi

# Set the library path
export LD_LIBRARY_PATH=$(pwd)/../target/release

# Run the test
cd java-sdk
mvn test --enable-native-access=ALL-UNNAMED
```

Or run the standalone main() smoke test:

```bash
mvn compile exec:java \
    -Dexec.mainClass="io.graphlite.sdk.SmokeTest" \
    -Dexec.classpathScope=test \
    --enable-native-access=ALL-UNNAMED
```

## Platform Support

| Platform | Status |
|---|---|
| Linux x86_64 | Supported |
| macOS aarch64 / x86_64 | Supported |
| Windows | Untested — library resolution includes `.dll`, but Rust FFI build on Windows is not verified |

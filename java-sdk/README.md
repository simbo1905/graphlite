# GraphLite Java SDK (Panama / FFM, Java 25)

High-level Java SDK for GraphLite built on Java's standard Foreign Function & Memory API (`java.lang.foreign`), without JNI.

## Goals

- Session-centric API (`GraphLite` + `Session`)
- Typed exceptions with native error codes
- Centralized Panama boundary (`io.graphlite.sdk.ffi.GraphLiteFFI`)
- Deterministic native ownership handling

## Module Layout

```text
java-sdk/
  src/main/java/io/graphlite/sdk/
    GraphLite.java
    Session.java
    Errors.java
    QueryResult.java
    ffi/
      GraphLiteFFI.java
      Layouts.java
```

## Prerequisites

1. Java 25
2. Maven 3.8+
3. Rust toolchain (to build `graphlite-ffi`)

## Build From Scratch

```bash
# 1) Build Rust FFI library
cargo build --release -p graphlite-ffi

# 2) Optional but recommended: pin explicit native library path
export GRAPHLITE_FFI_LIB="$(pwd)/target/release/libgraphlite_ffi.so"   # Linux
# export GRAPHLITE_FFI_LIB="$(pwd)/target/release/libgraphlite_ffi.dylib" # macOS

# 3) Build and install Java SDK to local Maven repo
cd java-sdk
mvn clean install
```

If `GRAPHLITE_FFI_LIB` is not set, the SDK tries:

1. `SymbolLookup.libraryLookup("graphlite_ffi", arena)`
2. common local paths such as `target/release` and parent variants
3. entries from `java.library.path`

## Quick Usage

```java
import io.graphlite.sdk.GraphLite;
import io.graphlite.sdk.QueryResult;
import io.graphlite.sdk.Session;

try (GraphLite db = GraphLite.open("./mydb");
     Session session = db.session("admin")) {
    session.execute("CREATE SCHEMA IF NOT EXISTS /demo");
    session.execute("SESSION SET SCHEMA /demo");
    session.execute("CREATE GRAPH IF NOT EXISTS social");
    session.execute("SESSION SET GRAPH social");

    session.execute("INSERT (:Person {name: 'Alice', age: 30})");
    QueryResult result = session.query("MATCH (p:Person) RETURN p.name AS name");

    for (var row : result) {
        System.out.println(row.get("name"));
    }
}
```

## Native Ownership Rules

- `graphlite_create_session` and `graphlite_query` return owned `char*`.
  - SDK copies to Java `String` and always calls `graphlite_free_string`.
- `graphlite_version` returns a static C string.
  - SDK reads it and does **not** free it.
- `graphlite_open` returns `GraphLiteDB*`.
  - SDK stores the handle and closes it with `graphlite_close` in `GraphLite.close()`.

## Error Model

`Errors` provides typed exceptions carrying both enum and raw native code:

- `ConnectionException`
- `SessionException`
- `QueryException`
- `SerializationException`
- `NativeLibraryException`

## Smoke Test

After building `graphlite-ffi`, run:

```bash
cd java-sdk
mvn test
```

## Platform Notes

- Linux and macOS are supported.
- On Windows, the SDK fails fast with a clear unsupported-platform message for this demo path.

# GraphLite Examples

This directory contains examples for using GraphLite across different programming languages and API levels.

## Directory Structure

```
examples/
├── rust/
│   ├── bindings/      # Core library examples (low-level API)
│   │   ├── simple_usage.rs
│   │   └── drug_discovery/
│   └── sdk/           # SDK examples (high-level ergonomic API)
│       ├── basic_usage.rs
│       └── drug_discovery/
├── python/
│   ├── bindings/      # FFI bindings examples (low-level)
│   │   ├── basic_usage.py
│   │   └── drug_discovery.py
│   └── sdk/           # High-level SDK examples
│       └── drug_discovery.py
└── java/
    ├── bindings/       # Java low-level bindings examples (JNA)
    │   └── BasicUsage.java
    └── sdk-panama/     # Java 25 high-level SDK examples (FFM/Panama)
        ├── DrugDiscovery.java
        └── BasicUsage.java
```

## Quick Start by Language

### Rust Examples

#### Core Library (Bindings)
Low-level API with direct access to GraphLite core:
```bash
# Simple usage
cargo run --example simple_usage -p graphlite

# Drug discovery (comprehensive)
cargo run --example drug_discovery -p graphlite
```

#### Rust SDK
High-level ergonomic API:
```bash
# Basic usage
cargo run --example basic_usage -p graphlite-rust-sdk

# Drug discovery
cargo run --example drug_discovery -p graphlite-rust-sdk
```

### Python Examples

#### FFI Bindings
Low-level Python bindings via ctypes:
```bash
cd examples/python/bindings

# Basic usage
python3 basic_usage.py

# Drug discovery
python3 drug_discovery.py
```

#### High-Level SDK
Session-centric Pythonic API:
```bash
cd examples/python/sdk

# Drug discovery
python3 drug_discovery.py
```

### Java Examples

#### Low-level Java bindings (JNA)

First, build and install the low-level Java bindings:
```bash
cd bindings/java
mvn install -DskipTests -Dmaven.javadoc.skip=true
```

Then run the low-level example:
```bash
cd examples/java/bindings
mvn clean compile exec:java
```

#### High-level Java SDK (Panama / Java 25)

```bash
# Build Rust FFI shared library
cargo build --release -p graphlite-ffi

# Build/install Java SDK module
cd java-sdk
mvn clean install

# Run high-level examples
cd examples/java/sdk-panama
mvn -q compile exec:java -Dexec.mainClass=DrugDiscovery
```

## Example Descriptions

### Simple/Basic Usage
Demonstrates fundamental operations:
- Opening a database
- Creating sessions
- Executing queries
- Basic CRUD operations

### Drug Discovery
Comprehensive pharmaceutical research example showing:
- **Domain**: Proteins (disease targets), Compounds (drugs), Assays (experiments)
- **Relationships**: TESTED_IN, MEASURES_ACTIVITY_ON, INHIBITS
- **Data**: IC50 measurements, clinical trial phases
- **Queries**: Filtering, traversal, aggregation, sorting

## API Level Comparison

| Aspect | Bindings (Low-Level) | SDK (High-Level) |
|--------|---------------------|------------------|
| **Abstraction** | Direct core access | Ergonomic wrapper |
| **Verbosity** | More code required | Concise |
| **Control** | Fine-grained | Simplified |
| **Use Case** | Advanced users | Recommended start |

## Language-Specific Documentation

- [Rust Bindings Examples](./rust/bindings/README.md)
- [Rust SDK Examples](./rust/sdk/README.md)
- [Python Bindings](./python/bindings/README.md)
- [Python SDK](./python/sdk/README.md)
- [Java SDK (Panama) Examples](./java/sdk-panama/README.md)

## Prerequisites

### Rust
```bash
cargo build --release
```

### Python
```bash
# Build FFI library
cargo build --release -p graphlite-ffi

# Install Python bindings
cd bindings/python
pip install -e .
```

### Java
```bash
# Build FFI library used by Java 25 FFM SDK
cargo build --release -p graphlite-ffi
```

## Contributing

When adding new examples:
1. Place in appropriate language/api-level directory
2. Follow naming convention: `example_name.{rs,py,java}`
3. Include inline documentation
4. Add entry to this README
5. Test across all supported platforms

## Related Documentation

- [GraphLite Core Documentation](../README.md)
- [Rust SDK Documentation](../graphlite-sdk/README.md)
- [Python Bindings Documentation](../bindings/python/README.md)
- [Java Bindings Documentation](../bindings/java/README.md)

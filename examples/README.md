# JDT Examples

This directory contains practical examples of JSON Document Transforms (JDT) demonstrating common transformation patterns.

## Examples

1. [Simple Merge](./01_simple_merge/) - Basic merge operation demonstrating default JDT behavior
2. [Environment Configuration](./02_environment_config/) - Transform development config to production settings
3. [Removing Sensitive Data](./03_remove_sensitive/) - Use `@jdt.remove` to strip sensitive fields
4. [Renaming Fields](./04_rename_fields/) - Use `@jdt.rename` to change key names
5. [JSONPath Selection](./05_jsonpath_selection/) - Use `@jdt.path` to target specific nodes

## Running Examples

Each example directory contains:
- `source.json` - The input JSON document
- `transform.json` - The JDT transform specification
- `expected.json` - The expected output
- `README.md` - Explanation of the transformation

To test an example:

```bash
cargo test --test ms_jdt_suite
```

Or use the library directly:

```rust
use jdt_codegen::transform::apply_transform;
use serde_json::json;

let source = json!({
    "name": "example",
    "version": "1.0.0"
});

let transform = json!({
    "version": "2.0.0",
    "environment": "production"
});

let result = apply_transform(&source, &transform).unwrap();
// Result: { "name": "example", "version": "2.0.0", "environment": "production" }
```

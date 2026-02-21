# Simple Merge Example

Demonstrates the default JDT behavior: merging the transform into the source.

## Source

A basic application configuration with name, version, description, and config object.

## Transform

Updates the version, modifies the port in config, adds a new logging property, and adds an author field.

## Result

The transform is merged into the source:
- `version` is updated from "1.0.0" to "1.0.1"
- `config.port` is updated from 3000 to 8080
- `config.logging` is added as a new property
- `config.host` remains "localhost" (not in transform)
- `author` is added as a new top-level property

This demonstrates that:
1. Matching keys are updated
2. New keys are added
3. Existing keys not in the transform are preserved
4. Nested objects are merged recursively

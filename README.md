# jdt-wasm

A Rust-based implementation of **JSON Document Transforms (JDT)** with WebAssembly (WASM) support, including:

- A Rust transformer (reference implementation)
- A JSONPath parser/evaluator (for `@jdt.path`)
- A WASM-facing wrapper crate (work in progress)
- A code generator that compiles transform files into executable code

The long-term goal mirrors the `jtd-wasm` pattern: compile a transform file into a compact plan or generated code, then optionally compile Rust to WASM for browser use.

## What is JDT?

JSON Document Transforms (JDT) is a specification for transforming JSON files using another JSON file as the transformation specification. JDT provides simple, intuitive transformations while keeping the transformation file as close to the original file as possible.

The default behavior is to merge the transformation file into the source file, with more advanced behavior available through specific transformation verbs:

- `@jdt.merge` - Merge values into the source
- `@jdt.replace` - Replace values in the source  
- `@jdt.remove` - Remove values from the source
- `@jdt.rename` - Rename keys in the source

JDT uses JSONPath expressions (relative to the current node) to target specific parts of the source document.

## Documentation

📚 **[View Documentation on GitHub Pages](https://simbo1905.github.io/jdt-wasm/)**

The [docs/](./docs/) folder contains the JDT specification documentation derived from the official Microsoft JDT wiki:

- [Home](./docs/Home.md) - Overview and introduction
- [Transform Verbs](./docs/Transform-Verbs.md) - Available transformation operations
- [Transform Attributes](./docs/Transform-Attributes.md) - Attributes like `@jdt.path` and `@jdt.value`
- [Default Transformation](./docs/Default-Transformation.md) - Default merge behavior
- [Merge Transformation](./docs/Merge-Transformation.md) - Explicit merge operations
- [Replace Transformation](./docs/Replace-Transformation.md) - Replace operations
- [Remove Transformation](./docs/Remove-Transformation.md) - Remove operations
- [Rename Transformation](./docs/Rename-Transformation.md) - Rename operations
- [Order of Execution](./docs/Order-of-Execution.md) - Transform execution order
- [JSONPath](./docs/JSONPath.md) - JSONPath syntax and usage

## Code Generation Specification

This implementation includes [JDT_CODEGEN_SPEC.md](./JDT_CODEGEN_SPEC.md), a language-independent specification for compiling JDT transforms into target-language source code. The specification defines:

- How to parse and validate transform files at compile time
- How to generate code that performs transformations without runtime parsing
- The intermediate representation (IR/plan) used during compilation
- Language-agnostic semantics that any code generator can follow

## Crate Structure

```
jdt-wasm/
├── jdt-codegen/                   # Core library (transform + jsonpath)
│   ├── src/
│   │   ├── jsonpath.rs            # JSONPath parser + selector evaluation
│   │   └── transform.rs           # JDT verbs + default transform semantics
│   └── tests/
│       └── ms_jdt_suite.rs        # Runs Microsoft JSON fixture suite
├── jdt-wasm-transformer/          # WASM-facing wrapper (placeholder)
├── docs/                          # JDT specification documentation
└── xmake.lua                      # Fetches fixture suites into .tmp/
```

## Testing

The `jdt-codegen` crate runs against Microsoft's official JSON test fixtures (automatically downloaded into `.tmp/`) and the JSONPath compliance test suite.

```bash
xmake run test_all
```

### Test Status

**Microsoft JDT Test Suite**: ✅ Passing  
**JSONPath Compliance Suite**: 255/703 tests passing (36%)

The JSONPath implementation supports:
- Basic property accessors (`$.field`, `$['field']`)
- Array indexing (`$[0]`, `$[-1]`)
- Filter expressions (`$[?(@.field == 'value')]`)
- Logical operators in filters (`&&`, `||`, `!`)

Currently unsupported (448 tests skipped):
- Wildcards (`*`, `$[*]`)
- Recursive descent (`..`)
- Array slices (`$[0:5]`, `$[::-1]`)

These features are tracked for future implementation.

## Requirements

- Rust (stable)
- [xmake](https://xmake.io/) - for test suite orchestration
- `curl` - to fetch upstream test suite

## License

This project is licensed under the MIT License. See [LICENSE](./LICENSE) for details.

## Attribution

The JDT specification and documentation in the [docs/](./docs/) folder are derived from the official **Microsoft JSON Document Transforms (JDT)** project:

- **Original Project**: [microsoft/json-document-transforms](https://github.com/microsoft/json-document-transforms)
- **Original Wiki**: [JDT Wiki](https://github.com/microsoft/json-document-transforms/wiki)
- **Copyright**: Copyright (c) Microsoft Corporation. All rights reserved.
- **License**: MIT License

This Rust/WASM implementation is an independent implementation of the JDT specification and is not affiliated with or endorsed by Microsoft Corporation.

### Microsoft JDT License

```
MIT License

Copyright (c) Microsoft Corporation. All rights reserved.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Contributing

Contributions are welcome! Please ensure that:

1. All tests pass (`xmake run test_all`)
2. Code follows Rust conventions and passes `cargo clippy`
3. New features include appropriate test coverage
4. Documentation is updated to reflect changes

## Roadmap

- [x] Basic JDT transform implementation
- [x] JSONPath parser and evaluator
- [ ] Complete WASM wrapper
- [ ] Code generation for multiple target languages
- [ ] Full JSONPath compliance
- [ ] Performance optimizations
- [ ] GitHub Pages documentation site

---

*For questions, issues, or contributions, please visit the [GitHub repository](https://github.com/YOUR_USERNAME/jdt-wasm).*

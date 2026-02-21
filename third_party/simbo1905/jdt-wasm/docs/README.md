<!--
This document is derived from the Microsoft JSON Document Transforms (JDT) Wiki.

Original Source: https://github.com/microsoft/json-document-transforms/wiki
Copyright (c) Microsoft Corporation. All rights reserved.
Licensed under the MIT License.

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
-->

# JDT WASM - JSON Document Transforms

A Rust-based implementation of **JSON Document Transforms (JDT)** with WebAssembly support.

[🏠 Main Repository](https://github.com/simbo1905/jdt-wasm) | [📦 Examples](https://github.com/simbo1905/jdt-wasm/tree/main/examples)

---

## What is JDT?

JSON Document Transforms (JDT) is a specification for transforming JSON files using another JSON file as the transformation specification. The default behavior is to merge the transformation into the source, with advanced operations available through transformation verbs.

**Transform Verbs:**
- `@jdt.merge` - Merge values into the source
- `@jdt.replace` - Replace values in the source  
- `@jdt.remove` - Remove values from the source
- `@jdt.rename` - Rename keys in the source

**Key Features:**
- JSONPath support for targeting specific nodes
- Recursive transformations for nested objects
- Filter expressions for conditional operations
- WebAssembly support for browser/Node.js usage

---

# JDT Specification Documentation

This documentation is derived from the official **Microsoft JSON Document Transforms (JDT)** project.

## Table of Contents

### Getting Started

- [Home](./Home.md) - Overview and introduction to JDT
- [Order of Execution](./Order-of-Execution.md) - How transforms are applied

### Core Concepts

- [Default Transformation](./Default-Transformation.md) - Understanding the default merge behavior
- [Transform Verbs](./Transform-Verbs.md) - Available transformation operations
- [Transform Attributes](./Transform-Attributes.md) - Using `@jdt.path` and `@jdt.value`
- [JSONPath](./JSONPath.md) - JSONPath syntax and usage in JDT

### Transformation Types

- [Merge Transformation](./Merge-Transformation.md) - Merging values into the source
- [Replace Transformation](./Replace-Transformation.md) - Replacing values in the source
- [Remove Transformation](./Remove-Transformation.md) - Removing values from the source
- [Rename Transformation](./Rename-Transformation.md) - Renaming keys in the source

## About This Documentation

This documentation is derived from the [Microsoft JSON Document Transforms Wiki](https://github.com/microsoft/json-document-transforms/wiki).

**Copyright**: Copyright (c) Microsoft Corporation. All rights reserved.  
**License**: MIT License  
**Original Source**: [https://github.com/microsoft/json-document-transforms](https://github.com/microsoft/json-document-transforms)

## Implementation

This documentation is part of the [jdt-wasm](../) project, a Rust-based implementation of JDT with WebAssembly support.

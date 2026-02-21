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

Attributes are advanced methods to specify behaviors that cannot be achieved through the default transformation. Any transformation can be an object containing valid attributes for that transformation. Attributes outside of transformations are not allowed and generate errors.

## Path

Use JSONPath syntax to navigate to the node where the transform should be applied.


| Use: | `"@jdt.path" : <Value>` (Case sensitive) |
| ---- |:----------------------------------------:|

Value must be a string with a valid JSONPath. Any other type generates an error. If the path does not match any nodes, the transformation is not performed.

All JDT Paths are relative to the node that they're in.

For more information on JSONPath syntax, see [here](http://goessner.net/articles/JsonPath/index.html)

## Value

The transformation value that should be applied.

| Use: | `"@jdt.value" : <Value>` (Case sensitive) |
| ---- |:-----------------------------------------:|

Value depends on the syntax of the transformation verb. See Transformation Verbs.
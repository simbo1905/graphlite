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

Transformations should be done in depth-first order to guarantee that everything is executed. Breadth-first order would have a slightly faster execution time as Remove transformations would potentially exclude other transformations from child nodes. For the same reason, this ordering could exclude transformations on lower level nodes that a user would like to execute.

In the same level, the order of priority for transformation is as follows:

Remove > Replace > Merge > Default > Rename

This order guarantees that explicitly named transforms execute first. It also guarantees that removals prevent unnecessary transformations from occurring.

## Processing Transform Files

To guarantee the order of execution defined previously, the transformed file is processed in the following order.

Starting from the root node:

1) Iterate through all non-*@jdt* verbs
    1.	Object
        1.	If it exists in the original file: step into the object and start from step 1
        2.	If it does not exist, enqueue a merge transformation
    2.	Other: Enqueue a `default` transformation
2) Enqueue all *@jdt* verbs
3) Process all the transformation in the queue, per the order of execution
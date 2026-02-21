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

If no behavior is specified in any JSON object, the default transformation is to merge the two files. The goal of JDT is to have a default behavior that satisfies most user scenarios. For more specifics on merging, see the Merge transformation.

#### Example

Source:
``` javascript
{
    "Version": 1,
    "Settings": {
        "Setting01" : "Default01",
        "Setting02" : "Default02"
    },
    "SupportedVersions" : [1, 2, 3]
}
```

Transform:
``` javascript
{
    "Version": 2,
    "Settings": {
        "Setting01" : "NewValue01",
        "Setting03" : "NewValue03"
    },
    "SupportedVersions" : [4, 5],
    "UseThis" : true
}
```

Result:
``` javascript
{
    // Overriden by the transformation file
    "Version": 2,
    "Settings": {
        // Overriden by the transformation file
        "Setting01" : "NewValue01",
        // Not present in the transformation file, unchanged
        "Setting02" : "Default02",
        // Added by the transformation file
        "Setting03" : "NewValue03"
    },
    // The array in the transformation file was appended
    "SupportedVersions" : [1, 2, 3, 4, 5],
    // Added by the transformation file
    "UseThis" : true
}
```
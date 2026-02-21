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

| Use: | `"@jdt.rename" : <Value>` (Case sensitive) |
| ---- |:------------------------------------------:|


| Value Type: | Behavior                        |
| ----------- | ------------------------------- |
| Primitive   | Not allowed. Generates an error |
| Object      | If the object contains JDT attributes, apply them. <br> See Attributes. If not, it must only contain key-value pairs where the key is the name of the node that should be renamed and the value is a string with the new name.
| Array       | Applies rename with each element of the array as the transformation value. <br> If the transformation value is an array, generate an error.

**Obs:** Renaming the root node is not allowed and will generate an error.

### Example

Source:
``` javascript
{
    "A" : {
        "A1" : 11,
        "A2" : {
            "A21" : 121,
            "A22" : 122
        }
    },
    "B" : [
        21,
        22
    ],
    "C" : 3
}
```

Transform:
``` javascript
{
    "@jdt.rename" : {
        "A" : "Astar",
        "B" : "Bstar"
    }
}
```

Result:
``` javascript
{
    // Does not alter result
    "Astar" : {
        "A1" : 11,
        "A2" : {
            "A21" : 121,
            "A22" : 122
        }
    },
    // Does not depend  on object type
    "Bstar" : [
        21,
        22
    ],
    // Does not alter siblings
    "C" : 3
}
```

## Path Attribute

The `@jdt.path` attribute can be used to specify a node to rename. Absolute or relative paths can be specified to the node. Renaming elements of arrays is not supported and should generate an error.

Source:
``` javascript
{
    "A" : {
        "RenameThis" : true
    },
    "B" : {
        "RenameThis" : false
    },
    "C" : [
        {
            "Name" : "C01",
            "Value" : 1
        },
        {
            "Name" : "C02",
            "Value" : 2
        }
    ]
}
```

Transform:
``` javascript
{
    "@jdt.rename" : {
        "@jdt.Path " : "$[?(@.Rename == true)]",
        "@jdt.Value" : "Astar"
    },
    "C" : {
        "@jdt.rename" : {
            "@jdt.path" : "@[*].Name",
            "@jdt.value" : "Nstar"
        }
    }
}
```

Result:
``` javascript
{
    // Only this node matches the path
    "Astar" : {
        "RenameThis" : true
    },
    "B" : {
        "RenameThis" : false
    },
    // Renaming nodes from an object 
    // in the array is allowed
    "C" : [
        {
            "Nstar" : "C01",
            "Value" : 1
        },
        {
            "Nstar" : "C02",
            "Value" : 2
        }
    ]
}
```
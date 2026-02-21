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

| Use: | `"@jdt.replace" : <Value>` (Case sensitive) |
| ---- |:-------------------------------------------:|


| Value Type: | Behavior                                                    |
| ----------- | ----------------------------------------------------------- |
| Primitive   | Replaces the current node with the given value |
| Object      | If the object contains JDT attributes, apply them. See Attributes. <br> If not, replaces the node with the given object.
| Array       | Applies merge with each element of the array as the transformation value. <br> If the transformation value is an array, replace the node with the given array.

### Example

Source:
``` javascript
{
    "A" : {
        "A1" : "11"
    },
    "B" : {
        "1B" : 12,
        "2B" : 22
    },
    "C" : {
        "C1" : 31,
        "C2" : 32
    }
}
```

Transform:
``` javascript
{
    "A": {
        "@jdt.replace": 1
    },
    "B": {
        "@jdt.replace": {
            "B1": 11,
            "B2": 12
        }
    },
    "C": {
        // Double brackets are needed to specify
        // the array as the transformation value
        "@jdt.replace": [[
            {
                "Value": 31
            },
            {
                "Value": 32
            }
        ]]
    }
}
```

Result:
``` javascript
{
    "A" : 1,
    "B" : {
        "B1" : 11,
        "B2" : 12
    },
    "C" : [
        {
            "Value": 31
        },
        {
            "Value": 32
        }
    ]
}
```

## Path Attribute

The `@jdt.path` attribute can be used to specify the absolute or relative path to the nodes that should be replaced. It can also be used to specify objects within arrays that should be replaced.

Source:
``` javascript
{
    "A" : {
        "A1" : 11,
        "A2" : "Replace"
    },
    "B" : [
        {
            "ReplaceThis" :true
        },
        {
            "ReplaceThis" : false
        }
    ]
}
```

Transform:
``` javascript
{
    "@jdt.replace" : {
        "@jdt.path" : "$.A.A2",
        "@jdt.value" : 12
    },
    "B" : {
        "@jdt.replace" : {
            "@jdt.path" : "@[?(@.ReplaceThis == true)]",
            "@jdt.value" : {
                "Replaced" : true
                }
        }
    }
}
```

Result:
``` javascript
{
    "A" : {
        "A1" : 11,
        "A2" : 12
    },
    "B" : [
        {
            // The entire object was replaced
            "Replaced" : true
        },
        {
            "ReplaceThis" : false
        }
    ]
}
```
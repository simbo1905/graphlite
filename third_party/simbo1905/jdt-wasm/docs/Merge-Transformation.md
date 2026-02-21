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

| Use: | `"@jdt.merge" : <Value>` (Case sensitive) |
| ---- |:-----------------------------------------:|


| Value Type: | Behavior                                                    |
| ----------- | ----------------------------------------------------------- |
| Primitive   | Replaces the value of the current node with the given value |
| Object      | Recursively merges the object into the current node. Keys that are not present in the source file will be added. <br> If the object contains JDT attributes, apply them. See Attributes.
| Array       | Applies merge with each element of the array as the transformation value. <br> In an explicit merge, if the transformation value should be the array, double brackets should be used (e.g. `[[<value>]]`). In a default transformation, this is not necessary.

**Obs:** If the transformation value does not match the source value for an already existing node, the transformation value will replace the existing one.

## Path Attribute

The `@jdt.path` attribute can be used if a specific node or multiple nodes should be changed. It can also be used to change nodes within arrays. See Attributes for more information.

Source:
``` javascript
{
    "A": {
        "TransformThis": true
    },
    "B": {
        "TransformThis": false
    },
    "C": {
    },
    "D": {
        "TransformThis": "WrongValue"
    },
    "E": {
        "TransformThis": false,
        "Items": [
            {
                "Value": 10
            },
            {
                "Value": 20
            },
            {
                "Value": 30
            }
        ]
    }
}
```

Transform:
``` javascript
{
    //Executes for all nodes on this level
    "@jdt.merge" : [{
        "@jdt.path" : "$.*",
        "@jdt.value" : {
            "Default" : 0
        }
    },
    //This only executes for matching nodes
    {
        "@jdt.path" : "$[?(@.TransformThis == true)]",
        "@jdt.Value" : {
            "Transformed" : true
        }
    }],
    "E": {
        // Accessing objects in array
        "@jdt.merge" : {
            "@jdt.path" : "$.Items[?(@.Value < 15)]",
            "@jdt.value" : {
                "Value" : 15,
                "Changed" : true
            }
        }
    }
}
```

Result:
``` javascript
{
    "A": {
        "TransformThis" : true,
        "Default" : 0,
        "Transformed" : true
    },
    "B": {
        "TransformThis": false,
        "Default" : 0
    },
    "C": {
        "Default" : 0
    },
    "D": {
        "TransformThis": "WrongValue",
        "Default" : 0
    },
    "E": {
        "TransformThis": false,
        "Items": [
            {
                "Value" : 15,
                "Changed" : true
            },
            {
                "Value": 20
            },
            {
                "Value": 30
            }
        ],
        "Default" : 0
    }
}
```

## Value Attribute

The `@jdt.value` attribute in a Merge is the only type that supports nested transformations. This means that transformations that should be executed in newly created or merged nodes can be added through this value.
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

| Use: | `"@jdt.remove" : <Value>` (Case sensitive) |
| ---- |:------------------------------------------:|


| Value Type:  | Behavior                                                    |
| ------------ | ----------------------------------------------------------- |
| String       | Removes the node with the given name from the current level |
| Boolean      | If true, remove all the nodes from the current level and sets value to null. If false, do nothing
| Number, null | Not allowed. Generates an error.
| Object       | If the object contains JDT attributes, apply them. See Attributes. <br> If not, generate error.
| Array        | Applies remove with each element of the array as the transformation value. <br> If the transformation value is an array, generate an error.


**Obs:** The `@jdt.value` attribute cannot be used with this transformation

### Example

Source:
``` javascript
{
    "A" : 1,
    "Astar" : 10,
    "B" : 2,
    "C" : {
        "C1" : 31,
        "C2" : 32
    },
    "D" : {
        "D1" : 41,
        "D2" : 42,
        "D3" : 43
    }
}
```

Transform:
``` javascript
{
    "@jdt.remove" : "Astar",
    "C" : {
        "@jdt.remove" : true
    },
    "D" : {
        "@jdt.remove" : ["D2", "D3"]
    }
}
```

Result:
``` javascript
{
    // Astar is completely removed
    "A" : 1,
    "B" : 2,
    // All nodes are removed
    "C" : null,
    "D" : {
        "D1" : 41
        // Multiple nodes were removed
    }
}
```

## Path Attribute

The `@jdt.path` attribute can be used to specify the absolute or relative path to the nodes that should be removed. It can also be used to remove elements from arrays. If the Path attribute is present, the Value attribute is not supported and is ignored if present in the transformation.

Source:
``` javascript
{
    "A" : {
        "RemoveThis" : true
    },
    "B" : {
        "RemoveThis" : false
    },
    "C" : {
        "C1" : 1,
        "C2" : {
            "C21" : 21
        }
    }
}
```

Transform:
``` javascript
{
    //Remove only matching nodes from this level
    "@jdt.remove" : {
        "@jdt.path" : "$[?(@.RemoveThis == true)]"
    },
    "C" : {
        //Specify a relative path to the node
        "@jdt.remove" : {
            "@jdt.path" : "@.C2.C21"
        }
    }
}
```

Result:
``` javascript
{
    "B" : {
        "RemoveThis" : false
    },
    "C" : {
        "C1" : 1,
        "C2" : {
        }
    }
}
```
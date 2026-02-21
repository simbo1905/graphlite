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

# JSON Document Transforms

The JDT language aims to provide simple, intuitive transformations for JSON files while keeping the transformation file as close to the original file as possible. JDT also provides more complex behavior through specific transformations with special syntax outlining the desired result.

Transformation options for JSON exist, but no definitive syntax is defined and most implementations are directly in Java or JavaScript (see [here](http://stackoverflow.com/questions/1618038/xslt-equivalent-for-json)). JDT is a specification that can be implemented in any language as it requires only two different JSON files to execute. The first implementation will be in C#, allowing for easy use in MSBuild projects.


## Summary

JSON document transformations seek to change a single JSON file (source) based on transformations specified in another JSON file (transform), generating a new JSON (result). The default behavior of JDT is to merge the transformation file into the source file. More advanced behavior can be specified by the user through the defined JDT syntax.

## Usage
.NET core projects support simple transformations of JSON files (such as an appSettings.json) depending on launch configurations (usually specified in a launchSettings.json). Similar to config transforms in web projects, an transformation file (such as appSettings.Production.json) can be specified with settings required only in the a specific environment. JDT defines more precisely the structure of the transformation file and allows for even more possibilities using a specific syntax.

Files with specific configurations would benefit from JDT by being able to automatically change their contents based on different runtime environments. Connections strings, for example, would benefit from this by using a production, testing or staging value. Sensitive information, such as passwords and secret keys can also be hidden in files that are not checked in.

For more detailed information, see:

* [[Transform Verbs]]

* [[Transform Attributes]]

* [JDT Schema](http://json.schemastore.org/jdt)
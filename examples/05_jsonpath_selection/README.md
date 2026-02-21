# JSONPath Selection Example

Demonstrates using `@jdt.path` to select and transform specific nodes matching a JSONPath query.

## Use Case

Target transformations to specific elements based on their properties:
- Update all items matching criteria
- Remove specific elements from arrays
- Merge properties into filtered subsets
- Bulk updates based on field values

## Source

Array of users with different roles:
```json
{
  "users": [
    {"id": 1, "name": "Alice", "role": "admin", "active": true},
    {"id": 2, "name": "Bob", "role": "user", "active": true},
    {"id": 3, "name": "Charlie", "role": "user", "active": false}
  ]
}
```

## Transform

Uses `@jdt.path` with a filter expression to select users with `role == 'user'`:
```json
{
  "users": {
    "@jdt.replace": {
      "@jdt.path": "$[?(@.role == 'user')]",
      "@jdt.value": {
        "role": "viewer"
      }
    }
  }
}
```

## Result

Only users with `role: "user"` are transformed:
- ✓ Alice (admin) - unchanged
- ✓ Bob (user) → role changed to "viewer"
- ✓ Charlie (user) → role changed to "viewer"

## JSONPath Features

JDT supports relative JSONPath expressions:
- `$` - current node
- `$.field` - select field
- `$[0]` - array index
- `$[*]` - all array elements
- `$[?(@.field == 'value')]` - filter by condition
- `$..field` - recursive descent

## Common Patterns

**Update all array items:**
```json
"@jdt.path": "$[*]"
```

**Select specific indices:**
```json
"@jdt.path": "$[0,2,4]"
```

**Filter by multiple conditions:**
```json
"@jdt.path": "$[?(@.active == true && @.role == 'user')]"
```

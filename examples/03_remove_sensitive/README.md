# Remove Sensitive Data Example

Demonstrates using `@jdt.remove` to strip sensitive information before sharing or logging.

## Use Case

Sanitize JSON data before:
- Logging to files or monitoring systems
- Sending to external APIs
- Sharing with third-party services
- Returning in public API responses

## Source

User data containing sensitive fields:
- Password hash
- API key
- Social Security Number
- Internal notes

## Transform

Uses `@jdt.remove` to specify arrays of keys to remove at different levels:
```json
{
  "user": {
    "@jdt.remove": ["password", "apiKey"],
    "profile": {
      "@jdt.remove": ["ssn"]
    }
  },
  "metadata": {
    "@jdt.remove": ["internalNotes"]
  }
}
```

## Result

Sensitive fields are removed:
- ✓ `user.password` removed
- ✓ `user.apiKey` removed
- ✓ `user.profile.ssn` removed
- ✓ `metadata.internalNotes` removed
- ✓ All other fields preserved

This allows you to:
1. Define sensitive field lists once
2. Apply consistently across your codebase
3. Keep the removal logic declarative
4. Audit which fields are being removed

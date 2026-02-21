# Rename Fields Example

Demonstrates using `@jdt.rename` to change key names in JSON documents.

## Use Case

Common scenarios for renaming fields:
- Converting snake_case to camelCase (or vice versa)
- Adapting legacy API responses to new naming conventions
- Normalizing data from multiple sources
- Migrating to new schema standards

## Source

User data with snake_case naming:
```json
{
  "user_id": 12345,
  "user_name": "johndoe",
  "email_address": "john@example.com",
  ...
}
```

## Transform

Uses `@jdt.rename` with old→new key mappings:
```json
{
  "@jdt.rename": {
    "user_id": "id",
    "user_name": "username",
    "email_address": "email",
    "created_date": "createdAt"
  },
  "account_settings": {
    "@jdt.rename": {
      "notification_enabled": "notifications",
      "theme_preference": "theme"
    }
  }
}
```

## Result

All specified keys are renamed:
- `user_id` → `id`
- `user_name` → `username`
- `email_address` → `email`
- `created_date` → `createdAt`
- `account_settings.notification_enabled` → `notifications`
- `account_settings.theme_preference` → `theme`

Values remain unchanged, only the keys are renamed.

## Benefits

1. Declarative field mapping
2. Works at any nesting level
3. Multiple renames in one transform
4. Safe: doesn't affect values or structure
5. Auditable: rename mappings are explicit

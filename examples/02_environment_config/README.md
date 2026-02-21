# Environment Configuration Example

Shows how to transform a development configuration into a production configuration.

## Use Case

Common scenario: You have a base configuration for development and need to override specific settings for production without duplicating the entire config file.

## Source

Development configuration with:
- Local database connection
- Cache disabled
- Debug logging
- No SSL

## Transform

Production overrides:
- Database points to production server with SSL
- Cache enabled with Redis backend
- Info-level logging
- Connection pooling configured

## Result

The production transform is merged into the development config:
- `database.host` changed to production server
- `database.name` changed to prod_db
- `database.ssl` enabled
- `database.port` preserved from source (5432)
- `database.poolSize` added
- `cache.enabled` set to true
- `cache.ttl` increased to 3600
- `cache.redis` configuration added
- `logging.level` changed to "info"
- `app` section unchanged

This pattern allows you to:
1. Maintain a single source config
2. Create small transform files per environment
3. Keep environment-specific changes visible and auditable

# Configuration Guide

## Environment Variables

Set these in your `.env` file:

```bash
export DB_URL="postgres://${DB_HOST:-localhost}:5432/app"
export SECRET_KEY='$(openssl rand -hex 32)'
export REDIS_URL="redis://127.0.0.1:6379/0"
export LOG_LEVEL="${LOG_LEVEL:-info}"
export BACKUP_CMD='pg_dump --format=custom "$DB_URL" | gzip > /tmp/backup_$(date +%Y%m%d).gz'
```

**Note:** Values containing `$()`, backticks, or `"` must be single-quoted.
Paths with spaces need escaping: `"/path/to/my\ files/"`.
## Deployment

Run `deploy.sh --env production` to push changes.

## Troubleshooting

If you see `error: connection refused`, check that the database is running.

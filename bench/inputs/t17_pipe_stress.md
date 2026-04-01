# Configuration Guide

## Environment Variables

Set these in your `.env` file:

```bash
export DB_URL="postgres://localhost:5432/app"
export SECRET_KEY='$(openssl rand -hex 32)'
export LOG_LEVEL=debug
```

## Deployment

Run `deploy.sh --env production` to push changes.

## Troubleshooting

If you see `error: connection refused`, check that the database is running.

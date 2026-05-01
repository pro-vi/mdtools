## API Guidelines

### API Rate Limiting

API clients must apply the shared request budget before retrying a call.

| Tier | Window | Limit |
| --- | --- | --- |
| Standard | 1 minute | 120 requests |
| Burst | 10 seconds | 30 requests |

```yaml
# API Rate Limiting
retry_after_header: X-RateLimit-Reset
```

### Pagination Rules

Paginated endpoints return opaque cursors and a `next` link.

## Operational Policies

### Data Retention

User activity logs are retained for 180 days.

### Support Escalation

Escalate repeated 429 reports to the on-call API owner.

## Appendix

### API Rate Limiting Exceptions

Internal load tests can request a temporary higher budget through the platform desk.

> Archive note: the old draft heading `### API Rate Limiting` in this note is historical and must stay here.

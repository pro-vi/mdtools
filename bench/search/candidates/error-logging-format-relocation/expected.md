# Operations Handbook

## Data Processing

### Error Logging Exceptions

Use this subsection only for exception-routing notes. It must stay under Data Processing.

## Reporting

### Error Logging Format

Every batch job must write one structured log entry when a retry is exhausted.

| Field | Required | Example |
|---|---|---|
| timestamp | yes | 2026-04-01T13:45:00Z |
| job | yes | nightly-invoice-sync |
| error_code | yes | RETRY_LIMIT |

```text
timestamp=2026-04-01T13:45:00Z job=nightly-invoice-sync error_code=RETRY_LIMIT attempt=3
```

- [ ] Include the owning service name.
- [ ] Include the retry counter.

### Report Generation Schedule

Daily reports are assembled at 03:00 UTC after processing windows close.

### Dashboard Metrics

Track failed batches, retry counts, and average processing time.

> Archive note: the deprecated heading `### Error Logging Format` appeared in the 2024 runbook and must not be edited.

## Appendix

Keep appendix notes in chronological order.

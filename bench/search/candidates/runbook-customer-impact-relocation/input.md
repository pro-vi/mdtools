# Incident Response Runbook

## Detection

Use this section while confirming whether an alert is customer-visible.

### Alert Criteria

- Error rate exceeds 2% for five minutes.
- Latency p95 stays above 1500 ms for ten minutes.
- Three or more regions report degraded health.

### Customer Impact Exceptions

Internal-only alerts, synthetic monitor failures, and staging regressions do not require a customer-facing impact summary.

### Customer Impact

Record the user-visible scope before the incident commander sends updates.

| Signal | Source | Owner |
|---|---|---|
| Affected tenants | analytics query | incident analyst |
| First bad deploy | deploy timeline | release captain |
| Data loss suspected | storage audit | service owner |

- [ ] Confirm affected tenant count.
- [ ] Capture first and last observed error times.
- [ ] Note whether data loss is suspected.

```text
Ticket template excerpt:
### Customer Impact
Paste the short customer-facing summary here.
```

### Initial Containment

Disable the feature flag only after rollback impact has been reviewed.

## Mitigation

Use this section to record operational actions taken during recovery.

### Rollback Decision

Rollback requires approval from the incident commander and release captain.

### Data Repair

Run repair jobs only after writes have been paused.

## Communications

Use this section for stakeholder and customer updates.

### Status Page Update

Publish an initial update within fifteen minutes of confirmed impact.

### Support Reply

Support should reuse the approved customer-facing summary.

> Archive note: older runbooks asked teams to fill `### Customer Impact` during closeout.

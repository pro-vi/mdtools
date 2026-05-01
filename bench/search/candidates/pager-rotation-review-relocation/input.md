# On-call Runbook

## Alert Triage

### Pager Rotation Exceptions

Use this subsection only for temporary coverage exceptions. It must stay under Alert Triage.

### Pager Rotation Review

Review active pager handoffs before acknowledging escalation queues.

| Signal | Owner | Action |
|---|---|---|
| stale-primary | Lead | confirm override |
| silent-secondary | SRE | page backup |

```yaml
template:
  heading: "### Pager Rotation Review"
  required: true
```

- [ ] Confirm primary and secondary responders.
- [ ] Note any swapped shifts in the incident log.

### Noise Suppression

Suppress duplicate monitor alerts only after the incident commander approves.

## Shift Handoff

### Shift Closeout Checklist

Record open incidents, paging overrides, and unresolved alerts before ending the shift.

### Follow-up Queue

Queue owners review stale handoff tasks at the start of the next shift.

> Archive note: the old heading `### Pager Rotation Review` was retired from the 2024 playbook and must stay quoted.

## Reference

Keep glossary and escalation policy links in this section.

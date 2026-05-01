# On-call Handoff

The live incident board below is copied into shift handoffs.

## Active Escalations

| Service | Severity | Owner | Runbook | Next step |
|---|---:|---|---|---|
| auth-api | 1 | Identity | [Auth](runbooks/auth.md) | restart `api\|worker` pair |
| billing-sync | 3 | Finance | [Billing](runbooks/billing.md) | monitor queue depth |
| search-index | 2 | Search | [Search](runbooks/search.md) | page search\|platform rotation |
| docs-preview | 4 | Docs | [Docs](runbooks/docs.md) | no action |

> Archived drill table, do not use:
> | Service | Severity | Owner | Runbook | Next step |
> | demo-api | 1 | Training | [Demo](runbooks/demo.md) | fake escalation |

```md
| Service | Severity | Owner | Runbook | Next step |
| staging-api | 1 | Sandbox | [Sandbox](runbooks/sandbox.md) | fake table in docs |
```

## Closed

No closed incidents for this handoff.

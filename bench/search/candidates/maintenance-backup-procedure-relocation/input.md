# Operations Runbook

# Operations

## Startup Sequence

- Start api
- Start worker

## Shutdown Sequence

- Stop worker
- Stop api

## Scheduled Tasks

| Task | Time |
| --- | --- |
| Rotate logs | 01:00 |
| Prune queues | 03:00 |

# Maintenance

## Backup Procedure

Follow this procedure before weekly releases.

| Step | Command |
| --- | --- |
| 1 | `snapshotctl create --scope app` |
| 2 | `snapshotctl verify --latest` |

```sh
snapshotctl create --scope app
snapshotctl verify --latest
```

## Restore Procedure

- Stop services
- Restore latest snapshot
- Start services

# Historical Notes

## Backup Procedure Archive

Older backups used `/srv/archive`.

```text
# Do not move this archived heading:
## Backup Procedure
```

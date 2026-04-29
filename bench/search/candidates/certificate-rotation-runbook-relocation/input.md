# Platform Runbook

## Security Procedures

### Key Inventory

Track active certificate IDs in the vault inventory.

### Certificate Rotation

Rotate production TLS certificates after renewal approval.

- [ ] Confirm the renewed bundle is present in vault.
- [ ] Notify the on-call lead before reload.

| Check | Owner |
|---|---|
| Expiry date verified | SRE |
| Reload window approved | Release manager |

```sh
certbot renew --deploy-hook "systemctl reload edge-proxy"
openssl x509 -in fullchain.pem -noout -dates
```

### Access Review

Review privileged access once per quarter.

## Operations Playbook

### Health Check

Run the edge probe and verify the public status page.

### Maintenance Window

Use the Tuesday 17:00 UTC maintenance window for low-risk reloads.

### Rollback Steps

Restore the previous bundle from vault and reload the proxy.

## Audit Notes

### Certificate Rotation Archive

Historical notes stay here for audit lookup.

```text
Do not move this archived outline:
### Certificate Rotation
- Retired cron job from 2023
```

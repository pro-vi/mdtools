# SOC 2 Evidence Collection Checklist

Quarterly evidence gathering. Stages marked `(in-scope)` are part of this audit
window; `(out-of-scope)` stages belong to the annual review and must not be
touched here. Check an item only once its evidence is attached.

## Access Controls (in-scope)

- [x] Export IAM role inventory
- [x] Collect MFA enrollment report
- [x] Attach quarterly access review
- [x] Snapshot privileged-group membership

## Change Management (in-scope)

- [x] Pull merged-PR approvals
- [x] Export deploy audit log
- [x] Attach change-freeze exceptions

## Monitoring (in-scope)

- [x] Export alert-coverage report
- [x] Attach on-call rotation
- [x] Snapshot log-retention config

## Vendor Review (out-of-scope)

- [ ] Collect MFA enrollment report
- [ ] Refresh vendor risk ratings
- [ ] Attach subprocessor list

## Notes

The "- [ ] Collect MFA enrollment report" item appears under both Access
Controls and the out-of-scope Vendor Review — only the Access Controls one is in
this window.

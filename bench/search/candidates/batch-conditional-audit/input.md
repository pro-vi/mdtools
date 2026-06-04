# SOC 2 Evidence Collection Checklist

Quarterly evidence gathering. Stages marked `(in-scope)` are part of this audit
window; `(out-of-scope)` stages belong to the annual review and must not be
touched here. Check an item only once its evidence is attached.

## Access Controls (in-scope)

- [x] Export IAM role inventory
- [ ] Collect MFA enrollment report
- [ ] Attach quarterly access review
- [ ] Snapshot privileged-group membership

## Change Management (in-scope)

- [ ] Pull merged-PR approvals
- [ ] Export deploy audit log
- [ ] Attach change-freeze exceptions

## Monitoring (in-scope)

- [ ] Export alert-coverage report
- [ ] Attach on-call rotation
- [ ] Snapshot log-retention config

## Vendor Review (out-of-scope)

- [ ] Collect MFA enrollment report
- [ ] Refresh vendor risk ratings
- [ ] Attach subprocessor list

## Notes

The "- [ ] Collect MFA enrollment report" item appears under both Access
Controls and the out-of-scope Vendor Review — only the Access Controls one is in
this window.

# Release Plan

## Phase 1 — Preparation

- [x] Write migration scripts
- [x] Update dependencies
- [ ] Freeze feature branches

## Draft Notes

These are temporary planning notes to be removed before release.

The release window is Q2. Coordinate with the platform team on
database migration timing. Ensure rollback procedures are documented
and tested before proceeding.

Key risks:
- Schema migration on large tables may cause downtime
- Third-party API deprecation timeline is unclear
- Mobile team needs two weeks notice for app store submission

Contact release-eng@example.com with questions.

## Phase 2 — Execution

- [x] Run migration on staging
- [ ] Deploy to staging
- [ ] Run smoke tests
- [ ] Deploy to production

## Phase 3 — Verification

- [ ] Monitor error rates
- [ ] Verify data integrity
- [ ] Sign off on release

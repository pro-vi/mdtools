# Migration Plan

## Phase 0 — Schema prep

- [x] Audit current schema
- [x] Create migration scripts
- [ ] Run dry migration on staging

## Notes

These notes are temporary and should be removed before Phase 1 begins.

The schema migration requires careful coordination between the frontend
and backend teams. Key considerations:

- Ensure backward compatibility during the transition period
- Monitor query performance after index changes
- Keep rollback scripts tested and ready
- Document all breaking changes in the changelog

Contact: platform-team@example.com for questions.

## Phase 1 — Data backfill

- [x] Prepare backfill scripts
- [ ] Execute backfill on production
- [ ] Verify data integrity post-backfill

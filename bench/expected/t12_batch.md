# Cross-region rollout checklist

This doc intentionally contains literal `[ ]` strings in prose, tables, comments,
and code. Only GFM task list items count as tasks.

## Archived notes

- [ ] Remove collation overrides
- [x] Delete legacy cleanup cron

## Phase 0 — SQL normalization

1. [x] Snapshot current schema
2. [x] Remove collation overrides
3. [x] Normalize default ORDER BY clauses
4. [x] Parser cleanup
   - [x] Token offsets
     - [x] Add regression test
   - [x] Range handling
     - [x] Add regression test
5. [x] Record rollout owner

> Review thread
> - [x] Confirm fallback locale mapping
> - [x] Capture before/after query plans

```sql
-- example only; not a real task
-- [ ] Remove collation overrides
UPDATE queries SET sql = REPLACE(sql, '[ ]', '[x]');
```

Scratchpad: not a task: `[ ]` this is inline code

bullet lookalike: • [ ] not a markdown task

em dash lookalike: — [ ] not a markdown task

## Phase 1 — Backfill

- [x] Build shadow indexes
- [x] Verify dry-run metrics
- [ ] Data cleanup
  - [ ] Add regression test
- [ ] Remove stale dashboards
- [ ] Étendre la couverture des tests

## Phase 2 — Cutover

- [x] Freeze schema changes
- [ ] Switch read traffic
- [ ] Remove collation overrides
- [ ] Publish release notes

## Appendix — Examples

| Item           | Status |
|----------------|--------|
| rollback plan  | [ ]    |

```bash
echo "- [ ] Freeze schema changes"
```

<!-- [ ] hidden template checkbox -->

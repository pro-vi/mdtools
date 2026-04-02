# Platform Migration Roadmap

**Status:** In Progress
**Last updated:** 2026-03-15

---

Legend: `[ ]` = pending, `[x]` = done. Note: `[ ]` in code blocks are not tasks.

## Phase 0: Schema Normalization (PLAT-100)

**Goal:** Dual-compatible schema without breaking current app.
**Oracle:** ORACLE_BASE
**Iteration limit:** 30

- [x] 0.1 Implement handler
  - Write unit tests
  - [x] Document API changes
    - [ ] Find all callsites

- [x] 0.2 Create interface
  - Find all callsites
  - Document API changes
  - Write unit tests

- [ ] 0.3 Add validation
  - Add retry logic

- [ ] 0.4 Add regression test
  - Handle edge cases

> Review thread
> - [ ] Confirm approach with platform team
> - [x] Capture design decision in ADR

**Checkpoint:** Oracle passes for Phase 0.
**Gate:** /gate

---

## Phase 1: Driver Abstraction (PLAT-101)

**Goal:** ORM uses either driver A or driver B at runtime.
**Oracle:** ORACLE_BASE
**Iteration limit:** 14

- [ ] 1.1 Configure settings
  - Add type annotations

- [x] 1.2 Configure settings
  - Add integration test
  - Find all callsites
  - Add retry logic

- [ ] 1.3 Add validation
  - Document API changes
  - [ ] Update config schema

- [ ] 1.4 Add regression test
  - Write unit tests
  - Add integration test
  - Add type annotations

- [ ] 1.5 Add validation
  - Add retry logic
  - Update config schema
  - [x] Add integration test

```typescript
// Example — not a task
const pending = items.filter(i => i.status === '[ ]');
// [ ] TODO: optimize this query
```

**Checkpoint:** Oracle passes for Phase 1.
**Gate:** /gate

---

## Phase 2: Service Extraction (PLAT-102)

**Goal:** Business logic in services, routes are thin wrappers.
**Oracle:** ORACLE_BASE
**Iteration limit:** 30

- [x] 2.1 Wire into existing
  - Write unit tests
  - Update imports
  - Add retry logic

- [ ] 2.2 Add error handling
  - Document API changes
  - Write unit tests
  - Update config schema

- [ ] 2.3 Create interface
  - Update config schema

- [x] 2.4 Review security
  - Update config schema
  - Write unit tests
  - Add retry logic

**Checkpoint:** Oracle passes for Phase 2.
**Gate:** /gate

---

## Phase 3: Provider Abstraction (PLAT-103)

**Goal:** Users provide keys, multiple providers supported.
**Oracle:** ORACLE_BASE
**Iteration limit:** 30

- [x] 3.1 Wire into existing
  - Document API changes

- [ ] 3.2 Add validation
  - Handle edge cases
  - Update config schema
  - Write unit tests

- [x] 3.3 Remove legacy code
  - Find all callsites

- [x] 3.4 Wire into existing
  - Handle edge cases
  - Add type annotations
  - Handle edge cases

- [ ] 3.5 Migrate data
  - Document API changes
  - Find all callsites

- [x] 3.6 Review security
  - Add integration test
  - Update config schema
  - Add type annotations
  - [x] Find all callsites

- [ ] 3.7 Implement handler
  - Update imports
  - Document API changes
  - Add type annotations

> Review thread
> - [ ] Confirm approach with platform team
> - [x] Capture design decision in ADR

**Checkpoint:** Oracle passes for Phase 3.
**Gate:** /gate

---

## Phase 4: Auth Abstraction (PLAT-104)

**Goal:** External auth optional, local mode uses fixed user.
**Oracle:** ORACLE_BASE
**Iteration limit:** 19

- [x] 4.1 Benchmark performance
  - Document API changes

- [ ] 4.2 Review security
  - Update config schema

- [x] 4.3 Configure settings
  - Write unit tests
  - Find all callsites
  - [ ] Add type annotations
    - [ ] Add type annotations

- [ ] 4.4 Migrate data
  - Update imports

- [ ] 4.5 Wire into existing
  - Document API changes
  - Handle edge cases

**Checkpoint:** Oracle passes for Phase 4.
**Gate:** /gate

---

## Phase 5: Desktop Shell (PLAT-105)

**Goal:** App builds as static SPA, desktop framework wraps it.
**Oracle:** ORACLE_BASE
**Iteration limit:** 27

- [ ] 5.1 Add validation
  - Update config schema
  - Add retry logic
  - Document API changes

- [ ] 5.2 Update documentation
  - Find all callsites
  - Document API changes
  - [x] Add type annotations

- [x] 5.3 Update documentation
  - Update config schema
  - [x] Add retry logic
    - [x] Add retry logic

- [ ] 5.4 Remove legacy code
  - Write unit tests
  - Add type annotations
  - [x] Handle edge cases

- [x] 5.5 Add regression test
  - Add type annotations
  - Find all callsites
  - Handle edge cases

```typescript
// Example — not a task
const pending = items.filter(i => i.status === '[ ]');
// [ ] TODO: optimize this query
```

**Checkpoint:** Oracle passes for Phase 5.
**Gate:** /gate

---

## Phase 6: Local Model Support (PLAT-106)

**Goal:** Local inference provider in desktop app.
**Oracle:** ORACLE_BASE
**Iteration limit:** 13

- [x] 6.1 Migrate data
  - Update imports
  - Handle edge cases
  - [x] Add type annotations

- [ ] 6.2 Migrate data
  - Find all callsites

- [ ] 6.3 Add regression test
  - Write unit tests
  - [x] Write unit tests

- [ ] 6.4 Wire into existing
  - Find all callsites
  - Handle edge cases
  - [ ] Add retry logic
    - [ ] Document API changes

- [ ] 6.5 Remove legacy code
  - Write unit tests
  - [ ] Document API changes
    - [ ] Find all callsites

**Checkpoint:** Oracle passes for Phase 6.
**Gate:** /gate

---

## Phase 7: Distribution (PLAT-107)

**Goal:** Publish to distribution channels.
**Oracle:** ORACLE_BASE
**Iteration limit:** 28

- [ ] 7.1 Add logging
  - Update imports
  - Find all callsites
  - Document API changes
  - [x] Add type annotations

- [ ] 7.2 Update documentation
  - Write unit tests
  - Find all callsites
  - Add type annotations

- [ ] 7.3 Migrate data
  - Add integration test
  - Write unit tests

- [ ] 7.4 Implement handler
  - Update imports
  - Add integration test

- [ ] 7.5 Review security
  - Find all callsites

- [ ] 7.6 Update documentation
  - Document API changes
  - [x] Update config schema

- [ ] 7.7 Configure settings
  - Update imports
  - Add retry logic

> Review thread
> - [ ] Confirm approach with platform team
> - [x] Capture design decision in ADR

**Checkpoint:** Oracle passes for Phase 7.
**Gate:** /gate

---

## Phase 8: Telemetry (PLAT-108)

**Goal:** Privacy-respecting usage analytics.
**Oracle:** ORACLE_BASE
**Iteration limit:** 19

- [ ] 8.1 Migrate data
  - Add type annotations
  - Update imports
  - [ ] Document API changes
    - [x] Write unit tests

- [ ] 8.2 Create interface
  - Add integration test
  - Document API changes
  - Add retry logic
  - [ ] Find all callsites
    - [ ] Add integration test

- [ ] 8.3 Configure settings
  - Add integration test
  - [x] Handle edge cases

- [ ] 8.4 Review security
  - Update imports
  - Document API changes
  - Find all callsites

**Checkpoint:** Oracle passes for Phase 8.
**Gate:** /gate

---

## Phase 9: Plugin System (PLAT-109)

**Goal:** Third-party extensions via sandboxed plugins.
**Oracle:** ORACLE_BASE
**Iteration limit:** 28

- [ ] 9.1 Add regression test
  - Update config schema
  - Find all callsites

- [ ] 9.2 Create interface
  - Add type annotations
  - Update config schema
  - Document API changes

- [ ] 9.3 Set up CI check
  - Update imports
  - Write unit tests
  - Update imports

- [ ] 9.4 Review security
  - Find all callsites
  - Update imports

- [ ] 9.5 Review security
  - Write unit tests
  - Add integration test
  - [ ] Handle edge cases

```typescript
// Example — not a task
const pending = items.filter(i => i.status === '[ ]');
// [ ] TODO: optimize this query
```

**Checkpoint:** Oracle passes for Phase 9.
**Gate:** /gate

---

## Phase 10: Collaboration (PLAT-110)

**Goal:** Real-time multi-user editing.
**Oracle:** ORACLE_BASE
**Iteration limit:** 25

- [ ] 10.1 Review security
  - Update config schema
  - Add integration test

- [ ] 10.2 Create interface
  - Write unit tests

- [ ] 10.3 Update documentation
  - Update config schema
  - Document API changes

- [ ] 10.4 Migrate data
  - Find all callsites
  - Add type annotations

- [ ] 10.5 Set up CI check
  - Find all callsites
  - Add type annotations

> Review thread
> - [ ] Confirm approach with platform team
> - [x] Capture design decision in ADR

**Checkpoint:** Oracle passes for Phase 10.
**Gate:** /gate

---

## Phase 11: Mobile Shell (PLAT-111)

**Goal:** Mobile app with offline-first sync.
**Oracle:** ORACLE_BASE
**Iteration limit:** 21

- [ ] 11.1 Migrate data
  - Handle edge cases

- [ ] 11.2 Add regression test
  - Handle edge cases
  - Find all callsites
  - Document API changes

- [ ] 11.3 Create test fixtures
  - Write unit tests
  - Update config schema
  - Handle edge cases
  - [ ] Update config schema

- [ ] 11.4 Update documentation
  - Add integration test
  - Document API changes
  - Add integration test

- [ ] 11.5 Create test fixtures
  - Document API changes
  - Update imports
  - [ ] Handle edge cases

- [ ] 11.6 Set up CI check
  - Add integration test
  - Handle edge cases
  - Document API changes

**Checkpoint:** Oracle passes for Phase 11.
**Gate:** /gate

---

## Appendix — Status Summary

| Phase | Status |
|-------|--------|
| Phase 0: Schema Normalization | done |
| Phase 1: Driver Abstraction | done |
| Phase 2: Service Extraction | done |
| Phase 3: Provider Abstraction | [ ] |
| Phase 4: Auth Abstraction | [ ] |
| Phase 5: Desktop Shell | [ ] |
| Phase 6: Local Model Support | [ ] |
| Phase 7: Distribution | [ ] |
| Phase 8: Telemetry | [ ] |
| Phase 9: Plugin System | [ ] |
| Phase 10: Collaboration | [ ] |
| Phase 11: Mobile Shell | [ ] |

<!-- [ ] hidden template checkbox — do not parse -->

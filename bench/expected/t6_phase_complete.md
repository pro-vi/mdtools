# Project Migration Progress

**Status:** In Progress
**Spec:** `specs/migration.md`
**Fidelity:** Standard

---

## Composite Oracle (every iteration)

```bash
check-all && test && build
```

Shorthand: **ORACLE_BASE**. Every task must pass this before marking complete.

---

## Phase 0: Schema Compatibility (PROJ-100)

**Goal:** Make schema dual-compatible without breaking current app.
**Oracle:** ORACLE_BASE
**Iteration limit:** 15

- [x] 0.1 App-side ID generation
  - Find all insert callsites
  - Add `id: generateId()` at every insert
  - Keep `.defaultRandom()` in schema as fallback

- [x] 0.2 Convert enums to text columns
  - Remove enum declarations — keep `as const` arrays and union types
  - Update column definitions to `text().$type<EnumType>()`
  - Add runtime guards at API boundaries

- [x] 0.3 Convert array columns to JSON
  - Change `text('tags').array()` to `jsonb('tags').$type<string[]>()`
  - Update insert/query patterns

- [ ] 0.4 Remove collation overrides
  - Remove `.collate('C')` from ordering columns
  - Add explicit test for sort ordering

**Checkpoint:** ORACLE_BASE passes. Existing app works identically.
**Gate:** /gate (lenses: /casting)

---

## Phase 1: DB Driver Abstraction (PROJ-101)

**Goal:** ORM uses either driver A or driver B, selected at runtime.
**Oracle:** ORACLE_BASE + `DRIVER=b test-cmd run src/lib/db/`
**Iteration limit:** 20

- [x] 1.1 Create driver interface
  - `src/lib/db/drivers/types.ts` — `DatabaseDriver` interface (getClient, runMigrations, close)
  - Export `DrizzleClient` type alias

- [x] 1.2 Wrap driver A in interface
  - `src/lib/db/drivers/driverA.ts` — wraps current connection logic
  - Zero behavior change

- [x] 1.3 Add driver B
  - `src/lib/db/drivers/driverB.ts` — implements DatabaseDriver
  - Stores data in filesystem or in-memory

- [x] 1.4 Runtime driver selection
  - `src/lib/db/drivers/index.ts` — factory based on env var
  - Default: driver A (backward compatible)

- [ ] 1.5 Schema initialization + upgrade
  - First-run: convert schema to DDL, execute against fresh DB
  - Upgrade path: store version in meta table, compare on startup
  - Test: fresh init, then simulate upgrade

- [ ] 1.6 Integration tests
  - Test recursive queries
  - Test transaction atomicity
  - Test ordering without collation overrides

- [ ] 1.7 E2E test page: `/test/driver-crud`
  - Initialize driver B in browser
  - Run: create → insert → read → delete
  - Sets `data-test-status`

**Checkpoint:** App works with both drivers.
**Gate:** /gate (lenses: /casting)

---

## Phase 2: Service Layer Extraction (PROJ-102)

**Goal:** Business logic lives in `src/lib/services/`, routes are thin wrappers.
**Oracle:** ORACLE_BASE
**Iteration limit:** 25

- [x] 2.1 Service layer structure
  - Create `src/lib/services/` with types for service context
  - `ServiceContext` type: `{ db: Client, userId: string }`

- [x] 2.2 Extract entity service
  - `src/lib/services/entities.ts` — CRUD operations
  - Move logic from API routes

- [x] 2.3 Extract document service
  - `src/lib/services/documents.ts` — update, get, project
  - Move logic from route handlers

- [ ] 2.4 Extract message service
  - `src/lib/services/messages.ts` — stitching, sync, compact
  - Move logic from server modules

- [ ] 2.5 Extract transform service
  - `src/lib/services/transforms.ts` — promotion, compilation
  - Move logic from route handlers

- [ ] 2.6 Wire services into local orchestrator
  - `src/lib/services/local.ts` — in local mode, call directly (no HTTP)
  - Factory: `createServiceContext(mode)` returns context

- [ ] 2.7 E2E test page: `/test/service-direct`
  - Call service layer directly (no fetch)
  - Compare results to expected shapes
  - Sets `data-test-status`

**Checkpoint:** All routes are thin wrappers.
**Gate:** /gate (lenses: /refactor) + /code-review

---

## Phase 3: Provider Abstraction (PROJ-103)

**Goal:** Users provide their own keys. Multiple providers supported.
**Oracle:** ORACLE_BASE
**Iteration limit:** 15
**Status:** COMPLETE

- [x] 3.1 Provider registry
  - `src/lib/providers/registry.ts` — register/get providers by name
  - Interface: `{ createModel, listModels, healthCheck }`

- [x] 3.2 Auth support
  - Two modes: API key or OAuth token
  - Token refresh with storage

- [x] 3.3 Credential management
  - `src/lib/stores/credentials.ts` — persisted store for keys + tokens
  - Settings UI component

- [x] 3.4 Model selection UI
  - `src/lib/components/ModelSelector.svelte` — provider + model picker
  - Wire into existing settings

- [x] 3.5 Local provider integration
  - Auto-detect local provider at localhost
  - Model listing from API
  - Test with mock server

- [x] 3.6 E2E test page: `/test/credentials`
  - **Always (structural):** credential persistence with fake key, provider registry assertions, UI rendering
  - **When keys set (live):** create provider → call API → assert response
  - Sets `data-test-status="pass"` when all applicable tests pass

**Checkpoint:** App works with either API key or OAuth token.
**Gate:** /gate (lenses: /casting, /svelte)
## Phase 4: Auth Abstraction (PROJ-104)

**Goal:** External auth is optional. Local mode uses a fixed local user.
**Oracle:** ORACLE_BASE
**Iteration limit:** 10

- [ ] 4.1 Auth adapter interface
  - `src/lib/auth/types.ts` — `AuthAdapter` interface
  - `src/lib/auth/external.ts` — wraps current auth logic
  - `src/lib/auth/local.ts` — returns fixed user

- [ ] 4.2 Wire auth adapter
  - Hooks select adapter based on env var
  - Default: external (backward compatible)

- [ ] 4.3 Remove auth gates in local mode
  - Service layer accepts userId from context
  - Page loaders work without external session

- [ ] 4.4 E2E test page: `/test/local-mode`
  - Boots with local auth + driver B + NO external auth vars
  - Full flow: app loads → sidebar → create entity → view loads → editor visible
  - Sets `data-test-status`

**Checkpoint:** App works with local auth mode.
**Gate:** /gate (lenses: /casting)

---

## Phase 5: Desktop Shell (PROJ-105)

**Goal:** App builds as static SPA. Desktop framework wraps it.
**Oracle:** ORACLE_BASE + `ADAPTER=static build-cmd` + `desktop-build` (after 5.3)
**Iteration limit:** 20

- [ ] 5.1 Add static adapter
  - Configure conditional adapter selection
  - SPA fallback for client-side routing

- [ ] 5.2 Handle server routes in static mode
  - Service layer called directly, not via fetch
  - Universal load functions delegate to client-loader

- [ ] 5.3 Client-side data loading
  - `src/lib/services/client-loader.ts` — mirrors server load functions
  - Universal loads delegate in static mode

- [ ] 5.4 Desktop initialization
  - Configure desktop framework
  - Point to static build output
  - Verify: desktop dev launches app window

- [ ] 5.5 Wire driver B in desktop context
  - Auto-detect desktop → use driver B with filesystem storage
  - Use framework API for app data directory

- [ ] 5.6 Wire credentials in desktop context
  - Framework secure store for encrypted key storage
  - Settings UI reads/writes to desktop store

- [ ] 5.7 Desktop packaging
  - Configure bundle settings
  - Produce distributable package

- [ ] 5.8 E2E test page: `/test/static-shell`
  - Verifies client-side data loading works
  - Creates entity via service layer, loads data, renders editor
  - Sets `data-test-status`

**Checkpoint:** Desktop build produces working distributable.
**Gate:** /gate + /code-review + /second-opinion

---

## Phase 6: Local Model Support (PROJ-106)

**Goal:** Local inference provider integration in desktop app.
**Oracle:** ORACLE_BASE
**Iteration limit:** 15

- [ ] 6.1 Local provider
  - `src/lib/providers/local.ts` — OpenAI-compatible provider
  - Base URL from env var, default `http://127.0.0.1:8000`
  - Health check: `GET /v1/models`

- [ ] 6.2 Local model detection
  - On app launch: probe local provider endpoints
  - Auto-register whichever responds into provider registry
  - Status indicator in UI

- [ ] 6.3 Model management UI
  - List models from active local provider
  - Recommended models with metadata

- [ ] 6.4 E2E test page: `/test/local-model`
  - **Always (structural):** provider registry assertions, model catalog loads, UI renders
  - **When provider available:** probe health, list models, send test prompt
  - Sets `data-test-status="pass"` when structural tests pass

- [ ] 6.5 End-to-end local flow verification
  - Driver B + local provider + local auth = fully local
  - Full flow: create entity → interact with local model → transform → edit
  - [CHECKPOINT] Human reviews full flow quality

**Checkpoint:** Fully functional local desktop app.
**Gate:** /gate --thorough + /code-review

---

## Checkpoint Notes Template

```
**Phase N Checkpoint:**
- **Date:** [ISO date]
- **Gate Report:** /gate -> [findings]
- **Reviews:** [/code-review, /second-opinion if applicable]
- **Oracle Result:** `[command]` -> PASS/FAIL
- **Issues Found:** [summary]
- **Resolution:** [what was fixed]
```

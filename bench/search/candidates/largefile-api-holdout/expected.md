# Public API Reference (v2)

Generated endpoint reference. `max_page_size` appears in every endpoint's Parameters table; the default 100 recurs across most endpoints — always scope a change to the intended endpoint.

## Service 01 — Gateway 1

Endpoints exposed by gateway 1. Each is independently rate-limited and paginated.

### PUT /v2/orders/{id}  (s1e1)

Operates on the orders resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/invoices/{id}  (s1e2)

Operates on the invoices resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/shipments/{id}  (s1e3)

Operates on the shipments resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/webhooks/{id}  (s1e4)

Operates on the webhooks resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/users/{id}  (s1e5)

Operates on the users resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/teams/{id}  (s1e6)

Operates on the teams resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/exports/{id}  (s1e7)

Operates on the exports resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/audits/{id}  (s1e8)

Operates on the audits resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/accounts/{id}  (s1e9)

Operates on the accounts resource for service 1. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 02 — Gateway 2

Endpoints exposed by gateway 2. Each is independently rate-limited and paginated.

### PATCH /v2/orders/{id}  (s2e1)

Operates on the orders resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/invoices/{id}  (s2e2)

Operates on the invoices resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/shipments/{id}  (s2e3)

Operates on the shipments resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/webhooks/{id}  (s2e4)

Operates on the webhooks resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/users/{id}  (s2e5)

Operates on the users resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/teams/{id}  (s2e6)

Operates on the teams resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/exports/{id}  (s2e7)

Operates on the exports resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/audits/{id}  (s2e8)

Operates on the audits resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/accounts/{id}  (s2e9)

Operates on the accounts resource for service 2. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 03 — Gateway 3

Endpoints exposed by gateway 3. Each is independently rate-limited and paginated.

### DELETE /v2/orders/{id}  (s3e1)

Operates on the orders resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/invoices/{id}  (s3e2)

Operates on the invoices resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/shipments/{id}  (s3e3)

Operates on the shipments resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/webhooks/{id}  (s3e4)

Operates on the webhooks resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/users/{id}  (s3e5)

Operates on the users resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/teams/{id}  (s3e6)

Operates on the teams resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/exports/{id}  (s3e7)

Operates on the exports resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/audits/{id}  (s3e8)

Operates on the audits resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/accounts/{id}  (s3e9)

Operates on the accounts resource for service 3. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 04 — Gateway 4

Endpoints exposed by gateway 4. Each is independently rate-limited and paginated.

### GET /v2/orders/{id}  (s4e1)

Operates on the orders resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/invoices/{id}  (s4e2)

Operates on the invoices resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/shipments/{id}  (s4e3)

Operates on the shipments resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/webhooks/{id}  (s4e4)

Operates on the webhooks resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/users/{id}  (s4e5)

Operates on the users resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/teams/{id}  (s4e6)

Operates on the teams resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/exports/{id}  (s4e7)

Operates on the exports resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/audits/{id}  (s4e8)

Operates on the audits resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/accounts/{id}  (s4e9)

Operates on the accounts resource for service 4. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 05 — Gateway 5

Endpoints exposed by gateway 5. Each is independently rate-limited and paginated.

### POST /v2/orders/{id}  (s5e1)

Operates on the orders resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/invoices/{id}  (s5e2)

Operates on the invoices resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/shipments/{id}  (s5e3)

Operates on the shipments resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/webhooks/{id}  (s5e4)

Operates on the webhooks resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/users/{id}  (s5e5)

Operates on the users resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/teams/{id}  (s5e6)

Operates on the teams resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/exports/{id}  (s5e7)

Operates on the exports resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/audits/{id}  (s5e8)

Operates on the audits resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/accounts/{id}  (s5e9)

Operates on the accounts resource for service 5. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 06 — Gateway 6

Endpoints exposed by gateway 6. Each is independently rate-limited and paginated.

### PUT /v2/orders/{id}  (s6e1)

Operates on the orders resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/invoices/{id}  (s6e2)

Operates on the invoices resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/shipments/{id}  (s6e3)

Operates on the shipments resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/webhooks/{id}  (s6e4)

Operates on the webhooks resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/users/{id}  (s6e5)

Operates on the users resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/teams/{id}  (s6e6)

Operates on the teams resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/exports/{id}  (s6e7)

Operates on the exports resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/audits/{id}  (s6e8)

Operates on the audits resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/accounts/{id}  (s6e9)

Operates on the accounts resource for service 6. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 07 — Gateway 7

Endpoints exposed by gateway 7. Each is independently rate-limited and paginated.

### PATCH /v2/orders/{id}  (s7e1)

Operates on the orders resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/invoices/{id}  (s7e2)

Operates on the invoices resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/shipments/{id}  (s7e3)

Operates on the shipments resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/webhooks/{id}  (s7e4)

Operates on the webhooks resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/users/{id}  (s7e5)

Operates on the users resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/teams/{id}  (s7e6)

Operates on the teams resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/exports/{id}  (s7e7)

Operates on the exports resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/audits/{id}  (s7e8)

Operates on the audits resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/accounts/{id}  (s7e9)

Operates on the accounts resource for service 7. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 08 — Gateway 8

Endpoints exposed by gateway 8. Each is independently rate-limited and paginated.

### DELETE /v2/orders/{id}  (s8e1)

Operates on the orders resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/invoices/{id}  (s8e2)

Operates on the invoices resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/shipments/{id}  (s8e3)

Operates on the shipments resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/webhooks/{id}  (s8e4)

Operates on the webhooks resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/users/{id}  (s8e5)

Operates on the users resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/teams/{id}  (s8e6)

Operates on the teams resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/exports/{id}  (s8e7)

Operates on the exports resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/audits/{id}  (s8e8)

Operates on the audits resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/accounts/{id}  (s8e9)

Operates on the accounts resource for service 8. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 09 — Gateway 9

Endpoints exposed by gateway 9. Each is independently rate-limited and paginated.

### GET /v2/orders/{id}  (s9e1)

Operates on the orders resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/invoices/{id}  (s9e2)

Operates on the invoices resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/shipments/{id}  (s9e3)

Operates on the shipments resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/webhooks/{id}  (s9e4)

Operates on the webhooks resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/users/{id}  (s9e5)

Operates on the users resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/teams/{id}  (s9e6)

Operates on the teams resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/exports/{id}  (s9e7)

Operates on the exports resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/audits/{id}  (s9e8)

Operates on the audits resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/accounts/{id}  (s9e9)

Operates on the accounts resource for service 9. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 10 — Gateway 10

Endpoints exposed by gateway 10. Each is independently rate-limited and paginated.

### POST /v2/orders/{id}  (s10e1)

Operates on the orders resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/invoices/{id}  (s10e2)

Operates on the invoices resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/shipments/{id}  (s10e3)

Operates on the shipments resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/webhooks/{id}  (s10e4)

Operates on the webhooks resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/users/{id}  (s10e5)

Operates on the users resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/teams/{id}  (s10e6)

Operates on the teams resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/exports/{id}  (s10e7)

Operates on the exports resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/audits/{id}  (s10e8)

Operates on the audits resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/accounts/{id}  (s10e9)

Operates on the accounts resource for service 10. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 11 — Gateway 11

Endpoints exposed by gateway 11. Each is independently rate-limited and paginated.

### PUT /v2/orders/{id}  (s11e1)

Operates on the orders resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/invoices/{id}  (s11e2)

Operates on the invoices resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/shipments/{id}  (s11e3)

Operates on the shipments resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/webhooks/{id}  (s11e4)

Operates on the webhooks resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/users/{id}  (s11e5)

Operates on the users resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/teams/{id}  (s11e6)

Operates on the teams resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/exports/{id}  (s11e7)

Operates on the exports resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/audits/{id}  (s11e8)

Operates on the audits resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/accounts/{id}  (s11e9)

Operates on the accounts resource for service 11. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 12 — Gateway 12

Endpoints exposed by gateway 12. Each is independently rate-limited and paginated.

### PATCH /v2/orders/{id}  (s12e1)

Operates on the orders resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/invoices/{id}  (s12e2)

Operates on the invoices resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/shipments/{id}  (s12e3)

Operates on the shipments resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/webhooks/{id}  (s12e4)

Operates on the webhooks resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/users/{id}  (s12e5)

Operates on the users resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/teams/{id}  (s12e6)

Operates on the teams resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/exports/{id}  (s12e7)

Operates on the exports resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/audits/{id}  (s12e8)

Operates on the audits resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/accounts/{id}  (s12e9)

Operates on the accounts resource for service 12. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 13 — Gateway 13

Endpoints exposed by gateway 13. Each is independently rate-limited and paginated.

### DELETE /v2/orders/{id}  (s13e1)

Operates on the orders resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/invoices/{id}  (s13e2)

Operates on the invoices resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/shipments/{id}  (s13e3)

Operates on the shipments resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/webhooks/{id}  (s13e4)

Operates on the webhooks resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/users/{id}  (s13e5)

Operates on the users resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/teams/{id}  (s13e6)

Operates on the teams resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/exports/{id}  (s13e7)

Operates on the exports resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/audits/{id}  (s13e8)

Operates on the audits resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/accounts/{id}  (s13e9)

Operates on the accounts resource for service 13. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 14 — Gateway 14

Endpoints exposed by gateway 14. Each is independently rate-limited and paginated.

### GET /v2/orders/{id}  (s14e1)

Operates on the orders resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/invoices/{id}  (s14e2)

Operates on the invoices resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/shipments/{id}  (s14e3)

Operates on the shipments resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/webhooks/{id}  (s14e4)

Operates on the webhooks resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/users/{id}  (s14e5)

Operates on the users resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/teams/{id}  (s14e6)

Operates on the teams resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/exports/{id}  (s14e7)

Operates on the exports resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/audits/{id}  (s14e8)

Operates on the audits resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/accounts/{id}  (s14e9)

Operates on the accounts resource for service 14. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 15 — Gateway 15

Endpoints exposed by gateway 15. Each is independently rate-limited and paginated.

### POST /v2/orders/{id}  (s15e1)

Operates on the orders resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/invoices/{id}  (s15e2)

Operates on the invoices resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/shipments/{id}  (s15e3)

Operates on the shipments resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/webhooks/{id}  (s15e4)

Operates on the webhooks resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/users/{id}  (s15e5)

Operates on the users resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/teams/{id}  (s15e6)

Operates on the teams resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/exports/{id}  (s15e7)

Operates on the exports resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/audits/{id}  (s15e8)

Operates on the audits resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/accounts/{id}  (s15e9)

Operates on the accounts resource for service 15. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 16 — Gateway 16

Endpoints exposed by gateway 16. Each is independently rate-limited and paginated.

### PUT /v2/orders/{id}  (s16e1)

Operates on the orders resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/invoices/{id}  (s16e2)

Operates on the invoices resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/shipments/{id}  (s16e3)

Operates on the shipments resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/webhooks/{id}  (s16e4)

Operates on the webhooks resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/users/{id}  (s16e5)

Operates on the users resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/teams/{id}  (s16e6)

Operates on the teams resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/exports/{id}  (s16e7)

Operates on the exports resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/audits/{id}  (s16e8)

Operates on the audits resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/accounts/{id}  (s16e9)

Operates on the accounts resource for service 16. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 17 — Gateway 17

Endpoints exposed by gateway 17. Each is independently rate-limited and paginated.

### PATCH /v2/orders/{id}  (s17e1)

Operates on the orders resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/invoices/{id}  (s17e2)

Operates on the invoices resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/shipments/{id}  (s17e3)

Operates on the shipments resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/webhooks/{id}  (s17e4)

Operates on the webhooks resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/users/{id}  (s17e5)

Operates on the users resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/teams/{id}  (s17e6)

Operates on the teams resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/exports/{id}  (s17e7)

Operates on the exports resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/audits/{id}  (s17e8)

Operates on the audits resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/accounts/{id}  (s17e9)

Operates on the accounts resource for service 17. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 18 — Gateway 18

Endpoints exposed by gateway 18. Each is independently rate-limited and paginated.

### DELETE /v2/orders/{id}  (s18e1)

Operates on the orders resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/invoices/{id}  (s18e2)

Operates on the invoices resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/shipments/{id}  (s18e3)

Operates on the shipments resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/webhooks/{id}  (s18e4)

Operates on the webhooks resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/users/{id}  (s18e5)

Operates on the users resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 250 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/teams/{id}  (s18e6)

Operates on the teams resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/exports/{id}  (s18e7)

Operates on the exports resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/audits/{id}  (s18e8)

Operates on the audits resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/accounts/{id}  (s18e9)

Operates on the accounts resource for service 18. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 19 — Gateway 19

Endpoints exposed by gateway 19. Each is independently rate-limited and paginated.

### GET /v2/orders/{id}  (s19e1)

Operates on the orders resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/invoices/{id}  (s19e2)

Operates on the invoices resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/shipments/{id}  (s19e3)

Operates on the shipments resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/webhooks/{id}  (s19e4)

Operates on the webhooks resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/users/{id}  (s19e5)

Operates on the users resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/teams/{id}  (s19e6)

Operates on the teams resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/exports/{id}  (s19e7)

Operates on the exports resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/audits/{id}  (s19e8)

Operates on the audits resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/accounts/{id}  (s19e9)

Operates on the accounts resource for service 19. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 20 — Gateway 20

Endpoints exposed by gateway 20. Each is independently rate-limited and paginated.

### POST /v2/orders/{id}  (s20e1)

Operates on the orders resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/invoices/{id}  (s20e2)

Operates on the invoices resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/shipments/{id}  (s20e3)

Operates on the shipments resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/webhooks/{id}  (s20e4)

Operates on the webhooks resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/users/{id}  (s20e5)

Operates on the users resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/teams/{id}  (s20e6)

Operates on the teams resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/exports/{id}  (s20e7)

Operates on the exports resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/audits/{id}  (s20e8)

Operates on the audits resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/accounts/{id}  (s20e9)

Operates on the accounts resource for service 20. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 21 — Gateway 21

Endpoints exposed by gateway 21. Each is independently rate-limited and paginated.

### PUT /v2/orders/{id}  (s21e1)

Operates on the orders resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/invoices/{id}  (s21e2)

Operates on the invoices resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/shipments/{id}  (s21e3)

Operates on the shipments resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/webhooks/{id}  (s21e4)

Operates on the webhooks resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/users/{id}  (s21e5)

Operates on the users resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/teams/{id}  (s21e6)

Operates on the teams resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/exports/{id}  (s21e7)

Operates on the exports resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/audits/{id}  (s21e8)

Operates on the audits resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/accounts/{id}  (s21e9)

Operates on the accounts resource for service 21. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 22 — Gateway 22

Endpoints exposed by gateway 22. Each is independently rate-limited and paginated.

### PATCH /v2/orders/{id}  (s22e1)

Operates on the orders resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/invoices/{id}  (s22e2)

Operates on the invoices resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/shipments/{id}  (s22e3)

Operates on the shipments resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/webhooks/{id}  (s22e4)

Operates on the webhooks resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/users/{id}  (s22e5)

Operates on the users resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/teams/{id}  (s22e6)

Operates on the teams resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/exports/{id}  (s22e7)

Operates on the exports resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/audits/{id}  (s22e8)

Operates on the audits resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/accounts/{id}  (s22e9)

Operates on the accounts resource for service 22. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 23 — Gateway 23

Endpoints exposed by gateway 23. Each is independently rate-limited and paginated.

### DELETE /v2/orders/{id}  (s23e1)

Operates on the orders resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/invoices/{id}  (s23e2)

Operates on the invoices resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/shipments/{id}  (s23e3)

Operates on the shipments resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/webhooks/{id}  (s23e4)

Operates on the webhooks resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/users/{id}  (s23e5)

Operates on the users resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/teams/{id}  (s23e6)

Operates on the teams resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/exports/{id}  (s23e7)

Operates on the exports resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/audits/{id}  (s23e8)

Operates on the audits resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/accounts/{id}  (s23e9)

Operates on the accounts resource for service 23. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

## Service 24 — Gateway 24

Endpoints exposed by gateway 24. Each is independently rate-limited and paginated.

### GET /v2/orders/{id}  (s24e1)

Operates on the orders resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/invoices/{id}  (s24e2)

Operates on the invoices resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/shipments/{id}  (s24e3)

Operates on the shipments resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/webhooks/{id}  (s24e4)

Operates on the webhooks resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### DELETE /v2/users/{id}  (s24e5)

Operates on the users resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 180 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### GET /v2/teams/{id}  (s24e6)

Operates on the teams resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 60 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### POST /v2/exports/{id}  (s24e7)

Operates on the exports resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 100 | items per page |
| rate_limit_rpm | integer | 90 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PUT /v2/audits/{id}  (s24e8)

Operates on the audits resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 200 | items per page |
| rate_limit_rpm | integer | 120 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

### PATCH /v2/accounts/{id}  (s24e9)

Operates on the accounts resource for service 24. Standard auth and pagination apply. Responses are cursor-paginated.

#### Parameters

| name | type | default | notes |
|------|------|---------|-------|
| cursor | string | null | pagination cursor |
| max_page_size | integer | 50 | items per page |
| rate_limit_rpm | integer | 150 | requests per minute |
| expand | string | none | comma-separated includes |

#### Responses

- `200` — success
- `429` — rate limited

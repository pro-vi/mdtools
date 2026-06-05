# Public API Reference (v2)

Generated endpoint reference. `max_page_size` appears in every endpoint's Parameters table; the default 100 recurs across most endpoints — always scope a change to the intended service and endpoint.

## Service 01 — Gateway 1

Endpoints exposed by gateway 1. Each is independently rate-limited and paginated.

### PUT /v2/orders/{id}

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

### PATCH /v2/invoices/{id}

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

### DELETE /v2/shipments/{id}

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

### GET /v2/webhooks/{id}

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

### POST /v2/users/{id}

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

### PUT /v2/teams/{id}

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

### PATCH /v2/exports/{id}

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

### DELETE /v2/audits/{id}

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

### GET /v2/accounts/{id}

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

### PATCH /v2/orders/{id}

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

### DELETE /v2/invoices/{id}

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

### GET /v2/shipments/{id}

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

### POST /v2/webhooks/{id}

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

### PUT /v2/users/{id}

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

### PATCH /v2/teams/{id}

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

### DELETE /v2/exports/{id}

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

### GET /v2/audits/{id}

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

### POST /v2/accounts/{id}

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

### DELETE /v2/orders/{id}

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

### GET /v2/invoices/{id}

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

### POST /v2/shipments/{id}

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

### PUT /v2/webhooks/{id}

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

### PATCH /v2/users/{id}

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

### DELETE /v2/teams/{id}

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

### GET /v2/exports/{id}

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

### POST /v2/audits/{id}

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

### PUT /v2/accounts/{id}

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

### GET /v2/orders/{id}

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

### POST /v2/invoices/{id}

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

### PUT /v2/shipments/{id}

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

### PATCH /v2/webhooks/{id}

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

### DELETE /v2/users/{id}

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

### GET /v2/teams/{id}

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

### POST /v2/exports/{id}

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

### PUT /v2/audits/{id}

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

### PATCH /v2/accounts/{id}

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

### POST /v2/orders/{id}

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

### PUT /v2/invoices/{id}

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

### PATCH /v2/shipments/{id}

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

### DELETE /v2/webhooks/{id}

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

### GET /v2/users/{id}

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

### POST /v2/teams/{id}

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

### PUT /v2/exports/{id}

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

### PATCH /v2/audits/{id}

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

### DELETE /v2/accounts/{id}

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

### PUT /v2/orders/{id}

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

### PATCH /v2/invoices/{id}

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

### DELETE /v2/shipments/{id}

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

### GET /v2/webhooks/{id}

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

### POST /v2/users/{id}

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

### PUT /v2/teams/{id}

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

### PATCH /v2/exports/{id}

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

### DELETE /v2/audits/{id}

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

### GET /v2/accounts/{id}

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

### PATCH /v2/orders/{id}

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

### DELETE /v2/invoices/{id}

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

### GET /v2/shipments/{id}

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

### POST /v2/webhooks/{id}

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

### PUT /v2/users/{id}

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

### PATCH /v2/teams/{id}

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

### DELETE /v2/exports/{id}

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

### GET /v2/audits/{id}

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

### POST /v2/accounts/{id}

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

### DELETE /v2/orders/{id}

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

### GET /v2/invoices/{id}

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

### POST /v2/shipments/{id}

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

### PUT /v2/webhooks/{id}

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

### PATCH /v2/users/{id}

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

### DELETE /v2/teams/{id}

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

### GET /v2/exports/{id}

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

### POST /v2/audits/{id}

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

### PUT /v2/accounts/{id}

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

### GET /v2/orders/{id}

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

### POST /v2/invoices/{id}

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

### PUT /v2/shipments/{id}

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

### PATCH /v2/webhooks/{id}

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

### DELETE /v2/users/{id}

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

### GET /v2/teams/{id}

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

### POST /v2/exports/{id}

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

### PUT /v2/audits/{id}

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

### PATCH /v2/accounts/{id}

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

### POST /v2/orders/{id}

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

### PUT /v2/invoices/{id}

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

### PATCH /v2/shipments/{id}

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

### DELETE /v2/webhooks/{id}

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

### GET /v2/users/{id}

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

### POST /v2/teams/{id}

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

### PUT /v2/exports/{id}

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

### PATCH /v2/audits/{id}

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

### DELETE /v2/accounts/{id}

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

### PUT /v2/orders/{id}

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

### PATCH /v2/invoices/{id}

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

### DELETE /v2/shipments/{id}

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

### GET /v2/webhooks/{id}

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

### POST /v2/users/{id}

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

### PUT /v2/teams/{id}

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

### PATCH /v2/exports/{id}

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

### DELETE /v2/audits/{id}

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

### GET /v2/accounts/{id}

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

### PATCH /v2/orders/{id}

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

### DELETE /v2/invoices/{id}

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

### GET /v2/shipments/{id}

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

### POST /v2/webhooks/{id}

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

### PUT /v2/users/{id}

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

### PATCH /v2/teams/{id}

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

### DELETE /v2/exports/{id}

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

### GET /v2/audits/{id}

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

### POST /v2/accounts/{id}

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

### DELETE /v2/orders/{id}

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

### GET /v2/invoices/{id}

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

### POST /v2/shipments/{id}

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

### PUT /v2/webhooks/{id}

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

### PATCH /v2/users/{id}

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

### DELETE /v2/teams/{id}

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

### GET /v2/exports/{id}

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

### POST /v2/audits/{id}

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

### PUT /v2/accounts/{id}

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

### GET /v2/orders/{id}

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

### POST /v2/invoices/{id}

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

### PUT /v2/shipments/{id}

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

### PATCH /v2/webhooks/{id}

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

### DELETE /v2/users/{id}

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

### GET /v2/teams/{id}

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

### POST /v2/exports/{id}

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

### PUT /v2/audits/{id}

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

### PATCH /v2/accounts/{id}

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

### POST /v2/orders/{id}

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

### PUT /v2/invoices/{id}

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

### PATCH /v2/shipments/{id}

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

### DELETE /v2/webhooks/{id}

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

### GET /v2/users/{id}

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

### POST /v2/teams/{id}

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

### PUT /v2/exports/{id}

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

### PATCH /v2/audits/{id}

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

### DELETE /v2/accounts/{id}

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

### PUT /v2/orders/{id}

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

### PATCH /v2/invoices/{id}

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

### DELETE /v2/shipments/{id}

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

### GET /v2/webhooks/{id}

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

### POST /v2/users/{id}

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

### PUT /v2/teams/{id}

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

### PATCH /v2/exports/{id}

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

### DELETE /v2/audits/{id}

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

### GET /v2/accounts/{id}

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

### PATCH /v2/orders/{id}

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

### DELETE /v2/invoices/{id}

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

### GET /v2/shipments/{id}

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

### POST /v2/webhooks/{id}

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

### PUT /v2/users/{id}

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

### PATCH /v2/teams/{id}

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

### DELETE /v2/exports/{id}

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

### GET /v2/audits/{id}

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

### POST /v2/accounts/{id}

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

### DELETE /v2/orders/{id}

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

### GET /v2/invoices/{id}

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

### POST /v2/shipments/{id}

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

### PUT /v2/webhooks/{id}

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

### PATCH /v2/users/{id}

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

### DELETE /v2/teams/{id}

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

### GET /v2/exports/{id}

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

### POST /v2/audits/{id}

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

### PUT /v2/accounts/{id}

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

### GET /v2/orders/{id}

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

### POST /v2/invoices/{id}

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

### PUT /v2/shipments/{id}

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

### PATCH /v2/webhooks/{id}

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

### DELETE /v2/users/{id}

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

### GET /v2/teams/{id}

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

### POST /v2/exports/{id}

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

### PUT /v2/audits/{id}

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

### PATCH /v2/accounts/{id}

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

### POST /v2/orders/{id}

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

### PUT /v2/invoices/{id}

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

### PATCH /v2/shipments/{id}

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

### DELETE /v2/webhooks/{id}

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

### GET /v2/users/{id}

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

### POST /v2/teams/{id}

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

### PUT /v2/exports/{id}

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

### PATCH /v2/audits/{id}

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

### DELETE /v2/accounts/{id}

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

### PUT /v2/orders/{id}

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

### PATCH /v2/invoices/{id}

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

### DELETE /v2/shipments/{id}

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

### GET /v2/webhooks/{id}

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

### POST /v2/users/{id}

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

### PUT /v2/teams/{id}

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

### PATCH /v2/exports/{id}

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

### DELETE /v2/audits/{id}

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

### GET /v2/accounts/{id}

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

### PATCH /v2/orders/{id}

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

### DELETE /v2/invoices/{id}

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

### GET /v2/shipments/{id}

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

### POST /v2/webhooks/{id}

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

### PUT /v2/users/{id}

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

### PATCH /v2/teams/{id}

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

### DELETE /v2/exports/{id}

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

### GET /v2/audits/{id}

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

### POST /v2/accounts/{id}

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

### DELETE /v2/orders/{id}

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

### GET /v2/invoices/{id}

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

### POST /v2/shipments/{id}

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

### PUT /v2/webhooks/{id}

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

### PATCH /v2/users/{id}

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

### DELETE /v2/teams/{id}

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

### GET /v2/exports/{id}

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

### POST /v2/audits/{id}

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

### PUT /v2/accounts/{id}

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

### GET /v2/orders/{id}

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

### POST /v2/invoices/{id}

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

### PUT /v2/shipments/{id}

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

### PATCH /v2/webhooks/{id}

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

### DELETE /v2/users/{id}

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

### GET /v2/teams/{id}

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

### POST /v2/exports/{id}

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

### PUT /v2/audits/{id}

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

### PATCH /v2/accounts/{id}

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

# Platform Architecture Notes

This document captures the structure of the product platform.
It is edited by backend, infrastructure, and support engineers.

## Current Architecture

The production platform is split into service, data, and operations layers.
This section describes what is deployed today.

### Service Topology

The API gateway routes traffic to the public API service.
The worker service consumes jobs from the queue.
The reporting service reads from replicas only.

### Database Tables

The accounts, users, and jobs tables are the active production tables.
The audit_events table is append-only and is retained for seven years.

### Request Flow

The gateway authenticates requests before forwarding them downstream.
Workers process asynchronous requests after the API writes job records.

## Data Layer

This section groups the documents that belong to storage ownership.

### Data Models

Domain models mirror persisted tables but keep API-only fields separate.
Model names should stay singular in prose and plural in table names.

### Database Schema

The application stores relational data in PostgreSQL.

```sql
CREATE TABLE accounts (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  name text NOT NULL
);

CREATE INDEX accounts_tenant_id_idx
  ON accounts (tenant_id);
```

Operational notes:

- Every multi-tenant table must include `tenant_id`.
- Schema changes require migration review before deployment.
- Backfills must be listed in the release checklist.

### Database Schema Exceptions

Legacy analytics tables do not use tenant scoping.
Do not use this subsection as the canonical schema reference.

### Repository Interfaces

Repositories hide SQL details from services.
Tests should cover repository behavior through fixture data.

## API Layer

The API layer contains handlers, serializers, and response policies.

### Public Endpoints

Public endpoints are versioned under `/v1`.
Breaking changes require a migration guide.

### Internal Endpoints

Internal endpoints are available only on the private network.
They must not be linked from customer-facing guides.

## Operations

Operations content supports incident response and release execution.

### Runbook References

```text
Archive note:
The old heading "### Database Schema" appears in release notes from 2024.
Do not edit this sample when reorganizing this architecture document.
```

### Release Checklist

- Confirm migrations were reviewed.
- Confirm dashboards were updated.
- Confirm support notes were published.

## Appendix

Appendix material is informational and is not part of the ownership map.

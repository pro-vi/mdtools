# Project Phoenix README

Project Phoenix contains the core services for the internal event platform.

## Getting Started

Clone the repository and install dependencies:

```bash
pip install -r requirements.txt
```

## Core Features

Project Phoenix provides real-time event routing and operational visibility.

### Data Ingestion

Data enters through Kafka streams and is normalized before downstream routing.

### Infrastructure Setup

Prepare the shared infrastructure before starting application containers.

| Component | Requirement |
|---|---|
| Docker | 24.x or newer |
| Compose | v2 plugin |
| Secrets | `.env.local` present |

```bash
docker compose pull
docker compose up -d postgres redis
```

- [ ] Confirm local ports are available.
- [ ] Confirm the staging namespace exists.

### API Gateway

The API gateway exposes versioned endpoints for partner services.

## Deployment Guide

Deployment is managed with Docker Compose in development and Helm in staging.

### Rollback Procedure

Rollback steps are documented separately for each release train.

## Advanced Topics

### Infrastructure Setup Exceptions

Record one-off setup exceptions here. Do not move this subsection.

> Archive note: the legacy heading `### Infrastructure Setup` appeared under Advanced Topics in the 2024 README and must not be edited.

### Performance Tuning

Use the load-test profile before changing worker concurrency.

## Changelog

See `CHANGELOG.md` for release notes.

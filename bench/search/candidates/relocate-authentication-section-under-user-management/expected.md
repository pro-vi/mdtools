# Project Phoenix API Documentation

## Architecture

## User Management

### Authentication
All API endpoints require OAuth 2.0 bearer tokens.

#### Token Acquisition
Client applications must obtain tokens via the `/oauth/token` endpoint.

#### Token Validation
Each request includes an `Authorization: Bearer <token>` header.

### User Registration
New users can sign up via the `/users` endpoint.

### User Profiles
User profiles are accessible at `/users/{id}`.

### Password Reset
Password reset flows are initiated via `/users/reset-password`.

## Rate Limiting
Requests are limited to 100 per minute per API key.

## Monitoring
Key metrics are exposed at `/metrics` in Prometheus format.

## Contributing
See `CONTRIBUTING.md` for development guidelines.

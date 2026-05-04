# Project API Guide

## Configuration

### Authentication

All API requests must include a valid bearer token in the `Authorization` header.

Example:
```http
GET /api/v1/resource HTTP/1.1
Authorization: Bearer YOUR_TOKEN_HERE
```

### Rate Limiting

The API enforces rate limiting:
- 100 requests per minute
- 1000 requests per hour

## Endpoints

### User

`GET /user`

Returns basic user information.

### Notifications

`POST /notifications`

Sends a notification to the user.

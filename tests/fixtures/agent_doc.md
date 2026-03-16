---
title: API Documentation
version: 3
status: draft
---

# API Reference

This document describes the method interface for our service.

## Authentication

Use the standard method to authenticate:

```python
client = APIClient(key="...")
client.authenticate()
```

Tokens expire after one hour. Use the refresh method to renew.

## Endpoints

### GET /users

Returns a list of users. This method supports pagination.

| Parameter | Type | Description |
|-----------|------|-------------|
| page | int | Page number |
| limit | int | Results per page |

### POST /users

Creates a new user. This method validates input before insertion.

### DELETE /users/:id

Removes a user. This method is irreversible.

## Error Handling

All errors follow the standard method for HTTP status codes.

See [RFC 7231](https://tools.ietf.org/html/rfc7231) for reference.
Also check our [internal docs][internal] for more detail.

[internal]: https://internal.example.com/errors "Error Guide"

## Changelog

- v3: Added DELETE method
- v2: Added pagination
- v1: Initial release

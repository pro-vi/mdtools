# API Reference

## Methods

### GET /items

Returns all items. Supports filtering by status.

### POST /items

Creates a new item.

## Configuration

Default port is 8080. Set `PORT` environment variable to override.

## Methods

### PUT /items/:id

Updates an existing item.

### DELETE /items/:id

Deletes an item. Requires admin privileges.

## Changelog

- v2: Added PUT and DELETE
- v1: Initial GET and POST

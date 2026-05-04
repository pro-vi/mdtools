# API Documentation

## Overview
This document describes the REST API endpoints for the Example API.

## Features
### Authentication
All endpoints require authentication via API key sent in the request header.
- API keys can be generated in the dashboard
- Keys are revoked after 90 days of inactivity

### Data Format
All responses return JSON.

## Security
### TLS
All endpoints require TLS 1.2 or higher.

## Usage
### Making Requests
Endpoints accept GET and POST.

### Rate Limits
See rate limit documentation.

## Error Codes
400: Bad Request
401: Unauthorized
500: Internal Server Error

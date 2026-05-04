# Project Oak README

## Overview
A microservice for managing user accounts and authentication.

## User Management

### Account Creation
Users can be created via POST /users. Required fields:
- username
- email
- password_hash

### Password Reset Flow
Users can request password resets via email.

## API Reference

### User Endpoints
GET /users/{id}
PUT /users/{id}

### System Endpoints
GET /health

### Authentication
All API endpoints require authentication via JWT tokens.

#### Token Generation
GET /auth/token with Basic Auth credentials returns a token.

#### Token Usage
Tokens are passed in the Authorization header.

## Deployment
Uses Docker containers orchestrated by Kubernetes.

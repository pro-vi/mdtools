# Project Phoenix v2.0.0

## Overview
A next-generation web application framework for rapid development of scalable, real-time applications.

## Installation

### Prerequisites
- Node.js v18+
- Docker (for local development)

### Quick Start
```bash
npm install phoenix-framework@latest
generate-phoenix-app my-new-app
cd my-new-app
npm start
```

## Development Setup

### IDE Configuration
Recommended: VS Code with Prettier and ESLint extensions.

### Logging
**Current Level:** DEBUG
**Log Format:** JSON

#### Log Levels
- `DEBUG`: Detailed debug information.
- `INFO`: Informational messages.
- `WARN`: Warnings that don't halt execution.
- `ERROR`: Errors that halt execution.
- `CRITICAL`: Critical failures.

#### Log Destinations
- `STDOUT`: Console output.
- `FILE`: Written to `/var/log/phoenix/app.log`.

### Debugging Tools
VS Code extensions for direct log stream viewing.

## Application Configuration

### Core Settings
`PORT=3000`
`NODE_ENV=development`

### Database Connection
`DB_HOST=localhost`
`DB_USER=phoenix_user`
`DB_PASS=secure_password`

### Feature Flags
`NEW_AUTH_ENABLED=true`
`OLD_API_COMPATIBILITY_MODE=false`

## Runtime Management

### Process Monitoring
Tools: PM2, nodemon.

### Scaling
Horizontal scaling via Kubernetes.

## Contributing
See `CONTRIBUTING.md` for guidelines.
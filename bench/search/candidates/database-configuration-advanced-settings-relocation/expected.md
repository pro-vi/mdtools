# Project README

## Introduction

This service runs the developer portal and its release dashboard.

## Setup

### Prerequisites

- Install Node.js 20
- Install PostgreSQL 15

### Installation

1. Clone the repository
2. Run `npm install`

## Usage

Start the local server:

```sh
npm run dev
# Old heading mention from a copied wiki page:
# ### Database Configuration
```

## Advanced Settings

### Database Configuration

Configure the application database before the first boot.

1. Set up the database connection:
   - Host: localhost
   - Port: 5432
   - Username: portal_admin
   - Password: change-me

2. Run the database migrations:

   ```sh
   npm run db:migrate
   npm run db:seed
   ```

3. Verify the database setup:
   - Check that the `users` table exists
   - Confirm the migration ledger is current

### Logging

- Enable detailed logging
- Set the log file path

### Database Configuration Archive

The archived production checklist is retained for audit reference only.

### Caching

- Enable caching
- Set cache expiration time

## Contributing

Please refer to the [contributing guidelines](CONTRIBUTING.md).

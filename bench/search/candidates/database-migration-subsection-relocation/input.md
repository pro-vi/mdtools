# 3. Migration Plan

## 3.1 Migration Steps

### 3.1.1 Pre-migration Backups

Backup all critical databases and application data. This is essential.

```sql
-- Example backup script:
BACKUP DATABASE production_db;
-- Do not move the Database Migration heading mentioned in this comment.
```

### 3.1.2 Database Migration

All database migration tasks will be executed by the database team.

- Pre-migration schema review
- Data validation checks
- Post-migration index rebuilding

| Table | Action |
|---|---|
| users | Migrate |
| products | Migrate |
| orders | Migrate |

```bash
# Database Migration
echo "Starting database migration process..."
```

## 3.2 Verification Steps

### 3.2.1 Smoke Tests

Basic connectivity and functionality checks.

### 3.2.2 Data Integrity Checks

Validate data consistency across migrated tables.

> Archive note: the phrase "Database Migration" appeared in the 2024 recovery drill notes, but that archived quote is not a section to move.

### 3.2.3 Database Migration Checklist

Use this checklist only after verification is complete.

- Confirm migration sign-off.
- Record the release ticket.

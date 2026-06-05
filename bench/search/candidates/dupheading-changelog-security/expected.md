# Changelog

All notable changes to this project are documented here. This project adheres
to [Semantic Versioning](https://semver.org/).

## [2.1.0] - 2026-05-28

### Added

- Streaming export endpoint for large result sets.
- `--dry-run` flag on the migration CLI.

### Fixed

- Race condition when two workers claimed the same job.

### Security

- Fixed CVE-2026-3310: authenticated path traversal in the export endpoint allowed reading arbitrary workspace files. Upgrade immediately.

## [2.0.0] - 2026-04-02

### Added

- Multi-tenant workspace isolation.

### Fixed

- Memory leak in the websocket reconnect loop.

### Security

- _No security advisories in this release._

## [1.9.0] - 2026-02-15

### Added

- Configurable retry backoff.

### Fixed

- Incorrect timezone handling in scheduled reports.

### Security

- _No security advisories in this release._

## Security Policy

Report vulnerabilities to security@example.com. We triage within two business
days. Do not file public issues for undisclosed vulnerabilities.

## Release Process

Each release section follows the Keep a Changelog template:

```markdown
## [X.Y.Z] - DATE

### Added
### Fixed
### Security
- _No security advisories in this release._
```

Copy the block above when cutting a new release, then fill in each subsection.

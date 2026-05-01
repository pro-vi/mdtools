# Developer Onboarding Guide

## Getting Started

```text
Historical outline from the 2024 setup guide:
### Installation
Do not move this archived heading mention.
```

### Installation

Install the local toolchain before the first build.

- Clone the repository.
- Run `pnpm install`.
- Copy `.env.example` to `.env.local`.

```bash
pnpm install
pnpm prepare
```

### First Build

Run the smoke build before opening a pull request.

- `pnpm build`
- `pnpm test`

## Prerequisites

### Runtime Versions

- Node.js 22.x
- pnpm 10.x

### Access Tokens

- Request the package registry token.
- Confirm read access to the staging project.

## Troubleshooting

### Installation Notes

Use this section only for common setup failures.

- Clear the package-store cache if installs hang.

```text
Archived troubleshooting heading:
### Installation
Do not move this archived note.
```

### Build Failures

- Re-run with `CI=1` to match the build server.

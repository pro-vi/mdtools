# Holdout Ledger

## 2026-07-01 - holdout_version 2

Reason: bench v3 prompt-neutrality rewrite and task provenance labeling.

- Task descriptions were neutralized to remove md-specific coaching and exact-count leaks.
- Every task gained `provenance` metadata so core and adversarially-mined splits are machine-readable.
- Input and expected-output bytes are unchanged.
- Prior holdout bundles stamped with `holdout_version` 1 are non-comparable with v3 headline results.

# Compare prompt guidance on one unchanged runner

**Date:** 2026-09-20
**Status:** Proposed
**Deciders:** Codex proposal, pending review

## Context

Changing the runner and its prompt together can make an outcome difference
impossible to attribute. Existing campaign identities already bind exact prompt
bytes, and ordinary reports deliberately reject mixed experiment identities.

## Decision

Keep one campaign controller, native sandbox, independent grader and immutable
attempt store. Share campaign admission policy without removing fresh checks at
the locked execution boundary. Use declarative configuration instead of copied
operator loops.

Compare `full_help` and `discovery` as two separate campaigns on the same runner
source. Require identical non-prompt identities and an unchanged no-md control.
Interleave adjacent pairs through the existing controller. A separate comparison
record owns pairing and analysis, not a second grading or admission policy.

Keep `full_help` as the default. Offline parity and scripted-provider tests do
not establish model quality or authorize subscription usage. The public diagnostic
contains 18 trials across T1/T2/T10; live consent and default adoption are separate
decisions. Preserve strict answer formatting, including failures caused by fences.

## Consequences and revisit trigger

Configuration and comparison support add maintained code even though duplicate
operator policy is removed. On-demand help can erase initial prompt savings or
reduce success. A changed runner, grader, task set or prompt requires a new frozen
comparison. Revisit the default only after audited paired outcomes are available;
three public tasks cannot establish general superiority.

## Enforcement

- `CampaignAdmission`, `configured_campaign` in `bench/harness.py`
- `PromptComparisonSpec`, `LiveRunGrant.assert_scope` in `bench/manifest.py`
- `run_comparison`, `report_comparison` in `bench/prompt_comparison.py`
- `bench/test_harness_refactor.py`, `bench/test_prompt_neutrality.py`,
  `bench/test_campaign_config.py`, `bench/test_prompt_comparison.py`

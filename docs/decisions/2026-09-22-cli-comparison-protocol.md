# Compare versioned CLI workflows under one prospective protocol

**Date:** 2026-09-22
**Status:** Proposed
**Amends:** [CLI evaluation authorities](2026-09-16-cli-evaluation-authorities.md)

## Context

The historical benchmark and the recovered harness do not judge identical
submissions. The historical runner could select intermediate tool JSON using
expected key names; the recovered runner grades only the terminal answer.
The completed public prompt diagnostic also exposed answers wrapped in fences.
Changing their recorded grades would mix protocols after observing outcomes.

The historical run metadata names a binary path without its digest or source
revision. The preserved legacy source is newer than that run. A reproducible
comparison can identify its versioned interfaces, but cannot certify an exact
reconstruction of the historical executable.

## Decision

Run legacy, compact, and no-md afresh on the same 24-task corpus, retaining the
documented T2 expected-answer correction. Keep Sonnet and Haiku campaigns
separate. Use common task facts, ordinary tools, limits, and final-answer rules.
Historical percentages remain context; fresh legacy and no-md outcomes are the
controls. Native file-tool comparisons remain outside the CLI study.

For JSON answer tasks, `parse_json_submission` accepts raw JSON or one complete
three-backtick fence with an optional lowercase `json` label. It consumes only
terminal bytes. It cannot select a value from prose, earlier tool output, or
expected answers. Strict value decoding and declared semantic comparisons remain
unchanged. Text and file families do not use this wrapper rule.

Record `SubmissionFormat` separately from `Grade`. Its policy snapshot is bound
to the frozen experiment; its syntax observation is verified against unchanged
terminal bytes. A valid wrapper does not imply a correct answer. This contract
uses a new grader version and experiment identity. Old receipts stay unchanged
and reproduce through their recorded source revision.

Require `CorePreparationConsent` before the controller reads nonpublic corpus
content. It binds the source commit, corpus objects, task scope, destinations,
approval quotation and exposure acknowledgment. Preparation permission and
`LiveRunGrant` are separate because the final campaign identity cannot exist
until authorized content has been hashed. Neither saved evidence nor a
configuration supplies new authority.

## Consequences

The study compares documented workflows under a common prospective protocol,
not exact historical execution or pure executable availability. Report all
planned outcomes, per-condition measurements, formatting, failures and paired
uncertainty. Missing quantities remain unknown; failed attempts retain cost.
No equivalence margin or favorable-result requirement is inferred.

The existing controller, native sandbox, immutable receipts and report path
remain in use. Changed sources or contracts require new prerequisite evidence.
Model quota, controller access to nonpublic content and result publication each
require their explicitly scoped authority. Prior corpus exposure limits claims
about unseen tasks.

## Revisit triggers

- A supported task requires a final-answer representation this grammar cannot
  express without losing required facts.
- A new comparison needs exact historical replication, native file tools, or
  another provider; identify that separate protocol before collecting data.
- A contract or measurement defect appears during a frozen study; preserve its
  records and establish a new experiment before any repair is evaluated.

## References

- `bench/neutral_scorer.py`: `parse_json_submission`, `SubmissionFormat`, `grade_submission`
- `bench/manifest.py`: `CorePreparationConsent`, `CampaignSpec`, `LiveRunGrant`
- `bench/harness.py`: `configured_campaign`, `AttemptStore`
- `bench/report.py`: `summarize`, `report_campaign`
- `bench/test_harness_json.py`: `test_terminal_json_raw_and_single_fence_equivalence`
- `bench/test_campaign_config.py`: `test_core_preparation_denied_before_source_read`
- `bench/test_report_inputs.py`: `test_condition_measurement_totals_include_failures_and_retries`
- `bench/CLI_EVAL.md`: methodology and historical diagnostic evidence

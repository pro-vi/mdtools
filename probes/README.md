# Probe Portfolio

This file is the canonical mutable status surface for mdtools probes. Individual
`PROTOCOL.md` and `RESULTS.md` files remain the evidence and provenance record;
do not rewrite them merely to normalize portfolio status. Use
[`TEMPLATE.md`](TEMPLATE.md) when starting a new evidence question.

Lifecycle status and evidence verdict are intentionally separate:

- **Protocol**: the question and decision rule are locked; no experiment has run.
- **Executed**: the authorized experiment ran and has a durable evidence artifact.
- **Closed**: the evidence produced the stated decision; any successor asks a
  different question.
- **Stopped before execution**: no result label was earned; the hypothesis remains
  untested even though the product-research line is no longer active.

## Portfolio

| Family | Probe | Lifecycle | Evidence verdict | Product disposition | Successor or reopening gate |
| --- | --- | --- | --- | --- | --- |
| Benchmark mechanism | [Failure taxonomy](taxonomy/PROTOCOL.md) | Executed / complete | [Exploratory mechanism evidence](../bench/RESULTS.md#mechanism-evidence-exploratory) over the locked v3 cells | Supports only the scoped weak/tool-poor mechanism claim | Re-run only for a newly preregistered corpus or estimand |
| Frontier regime | [Long documents](longdoc/PROTOCOL.md) | [Closed](longdoc/RESULTS.md#probe-verdict) | No correctness lift or byte-cost proxy advantage on either measured model | No supported frontier-native or long-document edge | Reopen only for a materially different preregistered regime |
| Frontier regime | [Transactional multifile drift](multifile/PROTOCOL.md) | [Closed](multifile/RESULTS.md#kill-condition-check) | Neither arm reached the required repeated reliability; no native+md edge | `--expect-etag` remains a scripting safety feature, not a demonstrated agent advantage | Reopen only with a new reliability mechanism and preregistered gate |
| Target identity | [Target-state etag](target_state_etag/PROTOCOL.md) | [Closed](target_state_etag/RESULTS.md#overall-verdict) | No tested stateless candidate graduated | Do not promote a production identity token | Continued through non-block and bounded-context falsification |
| Target identity | [Non-block target identity](non_block_target_identity/PROTOCOL.md) | [Closed](non_block_target_identity/RESULTS.md#promotion-outcome) | All four candidates were demoted across section, table, and task surfaces | Do not generalize block evidence into a non-block identity token | Continued through bounded positional context |
| Target identity | [Position-bound target identity](position_bound_target_identity/PROTOCOL.md) | [Closed](position_bound_target_identity/RESULTS.md#candidate-summaries-and-conclusion) | All four bounded-context candidates were demoted | Keep re-query plus fail-closed ambiguity as the production contract | Led to one final observability-limit protocol |
| Target identity | [Stateless observability limit](stateless_target_identity_observability/PROTOCOL.md) | **Stopped before execution** | Untested; no protocol result label was earned | No production identity architecture is authorized | Reopen only for a concrete consumer failure that re-query cannot resolve and an owner decision to compare stateful authority costs |

## Current Product Conclusion

The executed identity probes demote every tested stateless and bounded-context
candidate. They do not prove that durable identity is impossible. The final
closed-tuple observability hypothesis was specified but not executed.

The production contract therefore remains:

- `loc` identifies current structure, not durable lineage;
- `etag` fingerprints current target content, not durable lineage;
- ambiguous matches fail closed; and
- agents re-query and re-resolve after mutations.

Stateful identity work requires a concrete consumer failure that this contract
cannot resolve. Any reopening must compare explicit authority, lifecycle,
corruption, portability, cleanup, and concurrency costs before architecture.

## Probe And PR Discipline

One evidence question maps to one PR by default. Preserve safety boundaries as
separate commits and explicit authority transitions inside that PR:

1. **Protocol commit** — lock the hypothesis, minimum experiment, disconfirming
   evidence, result labels, and product decision. Do not author or execute the
   proposed runner in this phase.
2. **Source commit** — author the runner, manifest, and fixtures. Inspect every
   new executable path before authorizing import, compilation, or execution.
3. **Execution commit** — run only the inspected source, commit canonical results,
   and update this portfolio with the evidence verdict and product disposition.

A protocol-only PR is exceptional. It must either request an explicit owner
decision before source work or establish an independently reusable contract.
Otherwise keep the phases in one PR so the question, evidence, and disposition
land together.

Every probe PR must update this portfolio before merge. If work stops before
execution, record that lifecycle state here without synthesizing a `RESULTS.md`
or implying that the hypothesis was validated or falsified.

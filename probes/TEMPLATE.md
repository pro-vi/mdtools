# <Probe Name>

**Lifecycle:** Protocol
**Owner decision required before source work:** yes / no

## Product Grounding

- Product or architecture claim this probe tests:
- Concrete observation that makes the question load-bearing now:
- Decision that changes for each possible result:
- Why existing evidence does not already answer it:

If no concrete decision changes, stop: this is not yet a probe worth running.

## Hypothesis

State one falsifiable claim with a bounded subject and observation surface.

## Minimum Experiment

Describe the cheapest experiment capable of changing the decision. Name the
fixed cases, inputs, authority, measured outputs, and aggregation rule.

## Disconfirming Evidence

List the exact observations that falsify, narrow, or make the hypothesis
inconclusive. Operational failure is not supporting evidence.

## Authority And Safety Boundary

- Trusted inputs and their authentication:
- Forbidden authority and side channels:
- Filesystem, process, network, and credential boundary:
- Canonical artifact and non-mutating check command:

## Phase Boundary

### Protocol

Lock only this question, its decision rule, and its safety boundary. Do not
author or execute the proposed runner.

### Source

Author the runner, manifest, and fixtures in a separate commit. Inspect every
new executable path before authorizing import, compilation, or execution.

### Execution

Run only the inspected source. Record exact inputs, commands, hashes, results,
and independent verification in the canonical evidence artifact.

These phases normally remain commits in one PR. A protocol-only PR requires an
explicit owner decision or an independently reusable contract.

## Result Labels And Decision Rule

Define a closed result vocabulary and the mechanical condition for each label.
For every label, state the production action it authorizes or forbids.

## Promotion, Demotion, And Stop Path

- Promotion threshold:
- Demotion or falsification threshold:
- Inconclusive path:
- Stop-before-execution wording:
- Reopening gate:

Stopping before execution earns no result label and authorizes no production
architecture.

## Portfolio Update

Before merge, update [`README.md`](README.md) with lifecycle, evidence verdict,
product disposition, evidence links, and the successor or reopening gate.

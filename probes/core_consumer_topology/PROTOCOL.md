# Core Consumer Topology Probe

## Product Grounding

- Product or architecture claim this probe tests: the existing `mdtools`
  package can serve its CLI unchanged while its library target becomes a clean
  in-process dependency for Braid-shaped and reader-shaped consumers.
- Concrete observation that makes the question load-bearing now: outline,
  section, tasks, and `set-task` have been moved behind the library boundary,
  and the next extraction decision depends on whether this boundary works for
  all three consumers.
- Decision that changes for each possible result: continue extracting the
  remaining command semantics, narrow the library surface, or reject the
  topology before a release commitment.
- Why existing evidence does not already answer it: CLI tests do not prove
  before/after process parity, Braid's sibling path does not prove clean package
  adoption, and library unit tests do not prove a viewer can compose reads and
  candidate edits without filesystem authority.

## Hypothesis

The single-package library/binary topology simultaneously preserves the
existing `md` process contract, supports a clean Braid-shaped import adapter,
and supplies the first reader-shaped in-process read/edit slice without
CLI-only dependencies or filesystem writes.

## Minimum Experiment

Run three independent local lanes against fixed sources:

1. **CLI preservation:** build baseline commit `07eb509` and the inspected
   candidate source; compare exit code, stdout, stderr, JSON, and mutation bytes
   for schema, outline, section, tasks, task, and `set-task` cases. Run the full
   current Rust suite.
2. **Braid adoption:** package the candidate, extract only its Cargo package,
   and build a clean consumer that parses Markdown and immediately maps task
   status and optional spans into consumer-owned types. No sibling repository
   path is available inside the consumer root.
3. **Reader readiness:** against the same extracted package, build and run a
   consumer with default features disabled that parses a document, obtains the
   outline and a section, queries tasks, and produces a guarded `set-task`
   candidate while asserting that the original source and filesystem remain
   unchanged. Inspect its normal dependency tree for CLI-only crates.

Each lane records exact commands, exit codes, artifact hashes, and assertions.
Lane verdicts are independent; operational failure is `inconclusive`, never a
product pass or failure.

## Disconfirming Evidence

- CLI lane falsifies on any unexplained process-output, exit-code, diagnostic,
  candidate-byte, or final-file difference, or on any current Rust test failure.
- Braid lane falsifies if the clean consumer needs a mutable sibling checkout,
  imports CLI wire/process types, or cannot map immediately into consumer-owned
  status and optional provenance types.
- Reader lane falsifies if it must spawn `md`, enable the `cli` feature, access
  the filesystem through core operations, or cannot produce the expected
  candidate bytes from typed library operations.
- The reader dependency-closure claim narrows if a CLI-only crate appears in
  the consumer's normal dependency tree.
- Packaging falsifies if the produced package contains benchmark runs, probe
  corpora, inbox material, or other research-only artifacts.

## Authority And Safety Boundary

- Trusted inputs and their authentication: tracked source at baseline commit
  `07eb509`, inspected candidate source, tracked fixtures, Cargo manifests, and
  SHA-256 hashes recorded by the runner.
- Forbidden authority and side channels: no network, credentials, environment
  secrets, installed sibling source, untracked fixtures, external models, or
  mutation of Braid, pi-mdtools, or a future reader repository.
- Filesystem, process, network, and credential boundary: all builds and
  consumers run in new temporary directories; only the current repository is
  read; the probe writes no tracked source or canonical Markdown input.
- Canonical artifact and non-mutating check command: `RESULTS.md` is canonical;
  `python3 probe.py --check` validates its embedded manifest and verdicts
  without executing builds.

## Phase Boundary

### Protocol

This document locks the three lanes, their fixed baseline, safety boundary,
result vocabulary, and decision rule. No runner may execute before its source
is committed and inspected.

### Source

Author `probe.py` in a separate commit. Inspect every subprocess command,
temporary path, hash input, package-membership rule, and verdict branch before
execution.

### Execution

Execute only the inspected commit. Record the candidate commit, inputs,
commands, hashes, lane outcomes, failures, and overall result in `RESULTS.md`.

## Result Labels And Decision Rule

Per lane:

- `pass`: every required assertion completes and agrees with the lane contract.
- `partial`: all correctness assertions pass but a stated cleanliness or
  dependency-closure condition narrows the claim.
- `fail`: a product or compatibility assertion is falsified.
- `inconclusive`: setup or execution does not produce admissible evidence.

Overall:

- `foundation_validated`: all three lanes pass.
- `foundation_narrowed`: no lane fails or is inconclusive and at least one is
  partial.
- `foundation_rejected`: any lane fails.
- `foundation_inconclusive`: no lane fails, but at least one is inconclusive.

Only `foundation_validated` authorizes bulk extraction of the remaining command
semantics. `foundation_narrowed` authorizes only repairs within this foundation.
`foundation_rejected` forbids further extraction on this topology.

## Promotion, Demotion, And Stop Path

- Promotion threshold: `foundation_validated` on the inspected candidate.
- Demotion or falsification threshold: any `fail` lane.
- Inconclusive path: repair probe infrastructure without changing the locked
  hypothesis or score, then rerun.
- Stop-before-execution wording: “Stopped before execution; no topology result
  label was earned and no further extraction is authorized.”
- Reopening gate: a changed core contract, CLI protocol version, Cargo package
  layout, or materially different consumer requirement.

## Portfolio Update

Before completion, update [`../README.md`](../README.md) with the lifecycle,
overall verdict, per-lane results, product disposition, evidence links, and
reopening gate.

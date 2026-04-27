# Telemetry contract: `md tasks`

T8 telemetry is documentation-only. This file describes the recording shape a
future telemetry hook on `md tasks` would emit. Implementing the hook in `src/`
is admissible only when paired with a finding or auto-research candidate that
benefits from it, per the T8 spec.

## Why `md tasks` first

`md tasks --json` is the highest-traffic command in the benchmark and the agent
re-query pattern's primary input. Every mutation cycle (`md tasks` → mutate
→ `md tasks`) traverses it twice. If output stability, selector ambiguity, or
size-class behavior on this command drift, every dependent scorer cell is
affected before any other surface registers a defect. A telemetry contract here
is a no-regret hardening track per Pro's review.

## Recorded fields

Each invocation records a single stdout-only line (or per-file row when
multifile) with the following fields. T8 requires no external sink — the
contract is a shape, not a transport.

| field | type | description |
|---|---|---|
| `command` | string | Always `tasks`. |
| `args` | string | Minimum identifying args: `--status=<pending\|done>`, `--recursive`, `--json`, plus N file paths. The file paths themselves are not recorded; only the count. |
| `selector_type` | string | `none` for `md tasks`. The command operates on the whole file set; there is no loc/heading/search selector. Recorded for cross-command shape uniformity. |
| `input_size_class` | string | Bucketed total bytes across all input files: `xs` (≤1 KiB), `s` (≤16 KiB), `m` (≤256 KiB), `l` (≤4 MiB), `xl` (>4 MiB). |
| `input_files` | int | Count of input files actually read (post-`--recursive` resolution). |
| `output_type` | string | `json` when `--json`; `text` otherwise. Never `mutation` or `no-op` — `md tasks` is read-only. |
| `tasks_emitted` | int | Number of `TaskEntry` rows in the response (sum across files). Filtered by `--status` if supplied. |
| `error_class` | string\|null | `null` on exit-0; on non-zero exit, one of `parse` (markdown parse failure), `io` (file not found / permission denied / not utf-8), `cli` (clap arg validation), `internal` (uncaught). `address-unresolved` and `ambiguity` are not applicable to `md tasks` (no selector). |
| `elapsed_micros` | int | Wall-clock microseconds end-to-end. |

## Out of scope (T8)

- Per-task-entry detail (loc, status). The contract records aggregate counts,
  not row-level data; row-level data is what JSON output already carries.
- External sink. Stdout-only recording. A future loop may add an env-gated
  sink, but T8 forbids new CLI surfaces and a sink is borderline.
- Cross-command correlation IDs. Each invocation is independent.
- Bytes-of-output recording. Cross-executor incomparability (P3, archived)
  applies; a telemetry-internal `bytes_output` would inherit the same defect.

## Findings or candidates this would unblock

A future iteration may file findings against `md tasks` covering:

- **Selector ambiguity proxy** — `md tasks` has no selector, but downstream
  `md set-task <loc>` does. Aggregate `tasks_emitted` distribution per
  `input_size_class` would let auto-research detect tasks where the loc-space
  is unusually dense (loc-collision risk for `set-task`).
- **JSON instability proxy** — `tasks_emitted` should be deterministic per
  input bytes. A telemetry hook would let cross-seed PI runs detect
  non-determinism in the structural query path without re-deriving the
  comparison from raw stdout.
- **Re-query cycle attribution** — pairing `md tasks` telemetry with a
  subsequent `md set-task` telemetry would let the harness attribute
  re-query frequency mechanically rather than via prose ledger entries.

None of the above are findings yet. They are hypothesis hooks the contract
makes cheap to file when a real failing trace surfaces.

## Companion artifacts (when implemented)

- A unit test under `bench/test_harness_*.py` would pin the contract shape
  on a synthetic invocation. Added with the implementation, not before.
- `bench/probes/<finding>/` would carry the attribution probe demonstrating
  the failure class moved when telemetry is consulted.

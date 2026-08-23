# Core Consumer Topology Results

## Verdict

**foundation_validated**

- `cli_preservation`: **pass**
- `braid_adoption`: **pass**
- `reader_readiness`: **pass**

## Product Disposition

- The sampled CLI process contract is unchanged from `07eb509`:
  schema, outline, section, tasks, task, and `set-task` matched on exit code,
  stdout, stderr, JSON, and mutation bytes; the complete current Rust suite
  passed.
- A clean Braid-shaped consumer built from the 105-file Cargo package without a
  sibling checkout and immediately translated Markdown task status and source
  coordinates into consumer-owned types.
- A reader-shaped consumer built with default features disabled, had no Clap or
  Walkdir dependency, used outline/section/task queries in-process, and produced
  a guarded task-edit candidate without changing its source or filesystem.
- The foundation is sufficient to continue extracting Markdown operations.
  Rendering, viewer state, agent policy, and Braid workflow semantics remain
  outside mdtools.

## Reopening Gate

Reopen this topology result if the CLI protocol version changes, the Cargo
package layout changes, a consumer needs a separate release cadence, or a
future operation cannot preserve direct-core/CLI candidate parity.

## Evidence

- Candidate commit: `11127a8b3f8a1793b352a7699ca43d70463e6956`
- Baseline commit: `07eb509`
- The JSON below records the executed lane details and is checked without rerunning builds.

<!-- result-json:start -->
```json
{
  "baseline_commit": "07eb509",
  "candidate_commit": "11127a8b3f8a1793b352a7699ca43d70463e6956",
  "lanes": {
    "braid_adoption": {
      "consumer_exit": 0,
      "consumer_stderr": "",
      "consumer_stdout": "",
      "package_clean": true,
      "package_file_count": 105,
      "package_sha256": "ba5b6719b0500dc7e2c4387991e0df532ed279f1fb35098196e8d9b2396e262c",
      "verdict": "pass"
    },
    "cli_preservation": {
      "baseline_binary_sha256": "40892016bc110c68f8bdde765f70278731d7eefbfdf0fca3fef6e16d3bac6926",
      "candidate_binary_sha256": "11a2efc2d60e02b25723c7a8f450d0626e54a01d3ba468eed3d9c3e4bccc122a",
      "comparisons": [
        {
          "args": [
            "schema",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "outline",
            "--json",
            "tests/fixtures/basic.md"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "section",
            "Introduction",
            "tests/fixtures/basic.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "tasks",
            "tests/fixtures/progress_example.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "task",
            "9.0",
            "tests/fixtures/progress_example.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        }
      ],
      "mutation_match": true,
      "suite_exit": 0,
      "verdict": "pass"
    },
    "reader_readiness": {
      "cli_only_dependencies": [],
      "consumer_exit": 0,
      "consumer_stderr": "",
      "consumer_stdout": "",
      "package_clean": true,
      "package_sha256": "ba5b6719b0500dc7e2c4387991e0df532ed279f1fb35098196e8d9b2396e262c",
      "tree_exit": 0,
      "tree_stderr": "",
      "verdict": "pass"
    }
  },
  "overall": "foundation_validated",
  "schema_version": "core-consumer-topology.v1"
}
```
<!-- result-json:end -->

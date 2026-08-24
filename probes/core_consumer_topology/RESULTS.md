# Core Consumer Topology Results

## Verdict

**foundation_validated**

- `cli_preservation`: **pass**
- `braid_adoption`: **pass**
- `reader_readiness`: **pass**

## Product Disposition

- Every current CLI command family is unchanged from `07eb509` on its
  representative parity case: reads matched exit code, stdout, and stderr;
  mutations additionally matched final document bytes; the complete current
  Rust suite passed.
- A clean Braid-shaped consumer built from the published Cargo package without a
  sibling checkout and immediately translated Markdown task status and source
  coordinates into consumer-owned types.
- A reader-shaped consumer built with default features disabled, had no Clap or
  Walkdir dependency, used outline/section/task queries in-process, and produced
  a guarded task-edit candidate without changing its source or filesystem.
- The reusable single-document surface is complete for the current CLI:
  reads, guarded edit candidates, and relocation all run without process I/O.
  Rendering, viewer state, agent policy, and Braid workflow semantics remain
  outside mdtools.

## Reopening Gate

Reopen this topology result if the CLI protocol version changes, the Cargo
package layout changes, a consumer needs a separate release cadence, or a
future operation cannot preserve direct-core/CLI candidate parity.

## Evidence

- Candidate commit: `9849cd1ba0885895b7f3f6cd63b0a3f3b007c508`
- Baseline commit: `07eb509`
- The JSON below records the executed lane details and is checked without rerunning builds.

<!-- result-json:start -->
```json
{
  "baseline_commit": "07eb509",
  "candidate_commit": "9849cd1ba0885895b7f3f6cd63b0a3f3b007c508",
  "lanes": {
    "braid_adoption": {
      "consumer_exit": 0,
      "consumer_stderr": "",
      "consumer_stdout": "",
      "package_clean": true,
      "package_file_count": 116,
      "package_sha256": "c6ab65302af5c56b59fc674d29a9725b0428306acfce426cc4a8df1b2e4d8bbe",
      "verdict": "pass"
    },
    "cli_preservation": {
      "baseline_binary_sha256": "40892016bc110c68f8bdde765f70278731d7eefbfdf0fca3fef6e16d3bac6926",
      "candidate_binary_sha256": "597cbca9b7dc30ab40620750942ff3dd6ee7e40ac8a7a57e3fffca3dc1729951",
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
            "blocks",
            "tests/fixtures/basic.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "block",
            "0",
            "tests/fixtures/basic.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "search",
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
            "links",
            "tests/fixtures/basic.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "frontmatter",
            "tests/fixtures/frontmatter.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "collect",
            "tests/fixtures/frontmatter.md",
            "--field",
            "title",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "stats",
            "tests/fixtures/basic.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "table",
            "tests/fixtures/table.md",
            "--json"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true
        },
        {
          "args": [
            "table",
            "tests/fixtures/table.md",
            "--index",
            "1",
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
      "mutation_comparisons": [
        {
          "args": [
            "replace-block",
            "1",
            "doc.md",
            "--from",
            "payload.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "replace-block"
        },
        {
          "args": [
            "insert-block",
            "doc.md",
            "--after",
            "0",
            "--from",
            "payload.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "insert-block"
        },
        {
          "args": [
            "delete-block",
            "1",
            "doc.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "delete-block"
        },
        {
          "args": [
            "move-block",
            "0",
            "doc.md",
            "--after",
            "2",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "move-block"
        },
        {
          "args": [
            "replace-section",
            "A",
            "doc.md",
            "--from",
            "payload.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "replace-section"
        },
        {
          "args": [
            "delete-section",
            "A",
            "doc.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "delete-section"
        },
        {
          "args": [
            "move-section",
            "A",
            "doc.md",
            "--after",
            "B",
            "--keep-level",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "move-section"
        },
        {
          "args": [
            "set",
            "title",
            "doc.md",
            "new",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "set-frontmatter"
        },
        {
          "args": [
            "replace-table-row",
            "0",
            "0",
            "doc.md",
            "--from",
            "payload.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "replace-table-row"
        },
        {
          "args": [
            "insert-table-row",
            "0",
            "1",
            "doc.md",
            "--from",
            "payload.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "insert-table-row"
        },
        {
          "args": [
            "delete-table-row",
            "0",
            "0",
            "doc.md",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "delete-table-row"
        },
        {
          "args": [
            "set-task",
            "1.0",
            "doc.md",
            "--status",
            "done",
            "--json",
            "-i"
          ],
          "baseline_exit": 0,
          "candidate_exit": 0,
          "match": true,
          "name": "set-task"
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
      "package_sha256": "c6ab65302af5c56b59fc674d29a9725b0428306acfce426cc4a8df1b2e4d8bbe",
      "tree_exit": 0,
      "tree_stderr": "",
      "verdict": "pass"
    }
  },
  "overall": "foundation_validated",
  "schema_version": "core-consumer-topology.v1",
  "source_clean": true
}
```
<!-- result-json:end -->

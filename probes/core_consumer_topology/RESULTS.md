# Core Consumer Topology Results

## Verdict

**foundation_validated**

- `cli_preservation`: **pass**
- `packaged_api_adoption`: **pass**
- `library_only_composition`: **pass**

## Product Disposition

- Every current CLI command family is unchanged from `07eb509` on its
  representative parity case: reads matched exit code, stdout, and stderr;
  mutations additionally matched final document bytes; the complete current
  Rust suite passed.
- A clean packaged adapter built without a sibling checkout and immediately
  translated Markdown task status and source coordinates into consumer-owned
  types.
- A library-only consumer built with default features disabled, had no Clap or
  Walkdir dependency, composed direct structural queries in-process, and
  produced a guarded task-edit candidate without changing its source or
  filesystem.
- The reusable single-document surface is complete for the current CLI:
  reads, guarded edit candidates, and relocation all run without process I/O.
  Application-specific rendering, state, policy, and workflow semantics remain
  outside mdtools.

## Reopening Gate

Reopen this topology result if the CLI protocol version changes, the Cargo
package layout changes, a consumer needs a separate release cadence, or a
future operation cannot preserve direct-core/CLI candidate parity.

## Evidence

- Candidate commit: `53ef98ebd3e7fe20bebf47b8663a05d666bb63b2`
- Baseline commit: `07eb509`
- The JSON below records the executed lane details and is checked without rerunning builds.

<!-- result-json:start -->
```json
{
  "baseline_commit": "07eb509",
  "candidate_commit": "53ef98ebd3e7fe20bebf47b8663a05d666bb63b2",
  "lanes": {
    "cli_preservation": {
      "baseline_binary_sha256": "40892016bc110c68f8bdde765f70278731d7eefbfdf0fca3fef6e16d3bac6926",
      "candidate_binary_sha256": "7e86087cf02b342bec8525d9a391922f6c6b67e92df870827de4ce861de6f0c6",
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
    "library_only_composition": {
      "cli_only_dependencies": [],
      "consumer_exit": 0,
      "consumer_stderr": "",
      "consumer_stdout": "",
      "package_check_exit": 0,
      "package_check_stderr": "   Compiling proc-macro2 v1.0.106\n   Compiling unicode-ident v1.0.24\n   Compiling quote v1.0.45\n   Compiling serde_core v1.0.228\n   Compiling serde v1.0.228\n    Checking hashbrown v0.16.1\n   Compiling version_check v0.9.5\n    Checking equivalent v1.0.2\n   Compiling siphasher v1.0.2\n   Compiling libc v0.2.183\n   Compiling crossbeam-utils v0.8.21\n    Checking utf8parse v0.2.2\n    Checking typenum v1.20.1\n   Compiling fastrand v2.3.0\n   Compiling zmij v1.0.21\n    Checking cfg-if v1.0.4\n    Checking itoa v1.0.17\n    Checking colorchoice v1.0.5\n   Compiling zerocopy v0.8.48\n    Checking anstyle-query v1.1.5\n    Checking anstyle-parse v1.0.0\n    Checking tinyvec_macros v0.1.1\n   Compiling serde_json v1.0.149\n    Checking is_terminal_polyfill v1.70.2\n    Checking anstyle v1.0.14\n    Checking tinyvec v1.11.0\n   Compiling phf_shared v0.13.1\n    Checking clap_lex v1.1.0\n    Checking strsim v0.11.1\n   Compiling autocfg v1.5.0\n   Compiling generic-array v0.14.7\n    Checking memchr v2.8.0\n    Checking anstream v1.0.0\n   Compiling phf_generator v0.13.1\n   Compiling jetscii v0.5.3\n   Compiling phf_codegen v0.13.1\n   Compiling entities v1.0.1\n    Checking indexmap v2.13.0\n   Compiling heck v0.5.0\n   Compiling num-traits v0.2.19\n    Checking unicode-normalization v0.1.25\n   Compiling comrak v0.51.0\n    Checking clap_builder v4.6.0\n    Checking either v1.15.0\n    Checking same-file v1.0.6\n    Checking toml_write v0.1.2\n   Compiling rayon-core v1.13.0\n    Checking winnow v0.7.15\n    Checking walkdir v2.5.0\n    Checking phf v0.13.1\n    Checking caseless v0.2.2\n    Checking regex-syntax v0.8.10\n    Checking typed-arena v2.0.2\n    Checking ryu v1.0.23\n    Checking finl_unicode v1.4.0\n    Checking smallvec v1.15.1\n    Checking unsafe-libyaml v0.2.11\n    Checking rustc-hash v2.1.1\n    Checking plotters-backend v0.3.7\n    Checking ciborium-io v0.2.2\n    Checking itertools v0.10.5\n    Checking plotters-svg v0.3.7\n    Checking cast v0.3.0\n    Checking anes v0.1.6\n    Checking oorandom v11.1.5\n    Checking once_cell v1.21.4\n    Checking regex-automata v0.4.14\n   Compiling syn v2.0.117\n    Checking crossbeam-epoch v0.9.18\n    Checking crossbeam-deque v0.8.6\n    Checking criterion-plot v0.5.0\n    Checking regex v1.12.3\n    Checking cpufeatures v0.2.17\n    Checking is-terminal v0.4.17\n   Compiling serde_derive v1.0.228\n   Compiling zerocopy-derive v0.8.48\n   Compiling clap_derive v4.6.0\n    Checking crypto-common v0.1.7\n    Checking block-buffer v0.10.4\n    Checking digest v0.10.7\n    Checking sha2 v0.10.9\n    Checking clap v4.6.0\n    Checking plotters v0.3.7\n    Checking rayon v1.11.0\n    Checking toml_datetime v0.6.11\n    Checking serde_spanned v0.6.9\n    Checking serde_yaml v0.9.34+deprecated\n    Checking tinytemplate v1.2.1\n    Checking toml_edit v0.22.27\n    Checking toml v0.8.23\n    Checking half v2.7.1\n    Checking mdtools v0.1.0 (/private/var/folders/sw/zn4xtdd96nq_y2jvwzyx0vk80000gn/T/mdtools-core-consumer-tklg94rp/package/mdtools-0.1.0)\n    Checking ciborium-ll v0.2.2\n    Checking ciborium v0.2.2\n    Checking criterion v0.5.1\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.94s\n",
      "package_clean": true,
      "package_content_sha256": "d2c80831691c826707f82a24f89124ece1bf67fbc39dabef33a2bf68f31a7638",
      "tree_exit": 0,
      "tree_stderr": "",
      "verdict": "pass"
    },
    "packaged_api_adoption": {
      "consumer_exit": 0,
      "consumer_stderr": "",
      "consumer_stdout": "",
      "package_check_exit": 0,
      "package_check_stderr": "   Compiling proc-macro2 v1.0.106\n   Compiling unicode-ident v1.0.24\n   Compiling quote v1.0.45\n   Compiling serde_core v1.0.228\n   Compiling serde v1.0.228\n    Checking hashbrown v0.16.1\n   Compiling version_check v0.9.5\n    Checking equivalent v1.0.2\n   Compiling siphasher v1.0.2\n   Compiling libc v0.2.183\n   Compiling crossbeam-utils v0.8.21\n    Checking utf8parse v0.2.2\n    Checking typenum v1.20.1\n   Compiling fastrand v2.3.0\n   Compiling zmij v1.0.21\n    Checking cfg-if v1.0.4\n    Checking itoa v1.0.17\n    Checking colorchoice v1.0.5\n   Compiling zerocopy v0.8.48\n    Checking anstyle-query v1.1.5\n    Checking anstyle-parse v1.0.0\n    Checking tinyvec_macros v0.1.1\n   Compiling serde_json v1.0.149\n    Checking is_terminal_polyfill v1.70.2\n    Checking anstyle v1.0.14\n    Checking tinyvec v1.11.0\n   Compiling phf_shared v0.13.1\n    Checking clap_lex v1.1.0\n    Checking strsim v0.11.1\n   Compiling autocfg v1.5.0\n   Compiling generic-array v0.14.7\n    Checking memchr v2.8.0\n    Checking anstream v1.0.0\n   Compiling phf_generator v0.13.1\n   Compiling jetscii v0.5.3\n   Compiling phf_codegen v0.13.1\n   Compiling entities v1.0.1\n    Checking indexmap v2.13.0\n   Compiling heck v0.5.0\n   Compiling num-traits v0.2.19\n    Checking unicode-normalization v0.1.25\n   Compiling comrak v0.51.0\n    Checking clap_builder v4.6.0\n    Checking either v1.15.0\n    Checking same-file v1.0.6\n    Checking toml_write v0.1.2\n   Compiling rayon-core v1.13.0\n    Checking winnow v0.7.15\n    Checking walkdir v2.5.0\n    Checking phf v0.13.1\n    Checking caseless v0.2.2\n    Checking regex-syntax v0.8.10\n    Checking typed-arena v2.0.2\n    Checking ryu v1.0.23\n    Checking finl_unicode v1.4.0\n    Checking smallvec v1.15.1\n    Checking unsafe-libyaml v0.2.11\n    Checking rustc-hash v2.1.1\n    Checking plotters-backend v0.3.7\n    Checking ciborium-io v0.2.2\n    Checking itertools v0.10.5\n    Checking plotters-svg v0.3.7\n    Checking cast v0.3.0\n    Checking anes v0.1.6\n    Checking oorandom v11.1.5\n    Checking once_cell v1.21.4\n    Checking regex-automata v0.4.14\n   Compiling syn v2.0.117\n    Checking crossbeam-epoch v0.9.18\n    Checking crossbeam-deque v0.8.6\n    Checking criterion-plot v0.5.0\n    Checking regex v1.12.3\n    Checking cpufeatures v0.2.17\n    Checking is-terminal v0.4.17\n   Compiling serde_derive v1.0.228\n   Compiling zerocopy-derive v0.8.48\n   Compiling clap_derive v4.6.0\n    Checking crypto-common v0.1.7\n    Checking block-buffer v0.10.4\n    Checking digest v0.10.7\n    Checking sha2 v0.10.9\n    Checking clap v4.6.0\n    Checking plotters v0.3.7\n    Checking rayon v1.11.0\n    Checking toml_datetime v0.6.11\n    Checking serde_spanned v0.6.9\n    Checking serde_yaml v0.9.34+deprecated\n    Checking tinytemplate v1.2.1\n    Checking toml_edit v0.22.27\n    Checking toml v0.8.23\n    Checking half v2.7.1\n    Checking mdtools v0.1.0 (/private/var/folders/sw/zn4xtdd96nq_y2jvwzyx0vk80000gn/T/mdtools-core-consumer-tklg94rp/package/mdtools-0.1.0)\n    Checking ciborium-ll v0.2.2\n    Checking ciborium v0.2.2\n    Checking criterion v0.5.1\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.94s\n",
      "package_clean": true,
      "package_content_sha256": "d2c80831691c826707f82a24f89124ece1bf67fbc39dabef33a2bf68f31a7638",
      "package_file_count": 116,
      "verdict": "pass"
    }
  },
  "overall": "foundation_validated",
  "schema_version": "core-consumer-topology.v1",
  "source_clean": true
}
```
<!-- result-json:end -->

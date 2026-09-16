# Offline CLI evaluation recovery

U1 restores fresh synthetic execution; U2 adds independent grading and explicit
final submission; U3 supplies pinned CLI conditions, executable recipes and
direct compact/full replay. The public CLI stays synthetic-only. Its default command
creates synthetic fixtures, runs a trusted Python subprocess that edits one file,
then captures, grades and persists the result. It never opens the task registry.

## Install and verify

The inspected interpreter is Python 3.9.6 on macOS. The lock pins exact PyPI
pure-Python wheel hashes for markdown-it-py, mdurl, pytest and every pytest
dependency on Python 3.9. This is an initial lock of inspected versions. Portable
offline execution requires POSIX process groups. Public-fixture tests require jq.

```sh
python3 -m venv .venv
.venv/bin/python -m pip install --require-hashes --only-binary=:all: -r bench/requirements.lock
.venv/bin/python -m pytest -q bench/test_harness_run_artifacts.py bench/test_harness_task_split.py bench/test_native_runner.py bench/test_harness_json.py bench/test_neutral_scorer.py bench/test_prompt_neutrality.py bench/test_command_policy.py bench/test_manifest.py
.venv/bin/python -I bench/harness.py --offline-exercise
```

Direct script and module entry points select the same offline exercise by default.
An optional `--results-dir` must name a new controller-owned directory.
The output identifies its absolute artifact location. Temporary receipts use
`mdtools.cli-eval.offline/0`; they cannot serve as live-study evidence.

## Module owners and sources

H is `c93352002e3855527980dbdba0941c8099143e45`; P is
`c8e081301fe845d1d8b9bd9ac125d1dbf7c86fa7`.

- `harness.py` retains H task fields, staging/subprocess mechanics, capture and
  artifact-writing intent (H lines 1227, 1398, 1683). Relative paths retain the
  complete path below the supplied root. U2 captures explicit final submissions
  and dispatches independent grading only after normal completion.
- `neutral_scorer.py` owns `StructuralDiffPolicy`, independent comparisons
  and shared answer contracts. It adapts H scoring intent (lines 485, 668 and
  independent heading primitives), replacing lossy source extraction and
  treatment diagnostics. It imports no harness or subprocess.
- `manifest.py` recovers H `v3_manifest.py` hash primitives with canonical
  serialization and a synthetic-only specification. Separate task, harness and
  grader SHA256 fields bind definition and implementation content without mixing
  their meanings. U4 still owns complete experiment identity and validated records.
- `command_policy.py` retains P's complete ordinary toolkit plus jq, exact
  planned condition names and explicit trusted synthetic argv admission.
  U3 decodes each pinned producer's schema, resolves toolkit targets, and stages
  the exact binary/stub. U5 still owns actual native containment.

No Pi/OAI/multifile imports, provider launch branches, competing correctness
booleans, quarantine, task generation, historical report policy or saved-run
dependency remains. Corpus files are retained unchanged.

## Task and filesystem contract

`run_agent` receives a task, disjoint fixture/expected/controller roots,
explicit synthetic argv and a positive finite deadline. Input/support references
are canonical relative POSIX paths. The first input is the final file for file
kinds; multiple inputs retain distinct relative paths for extraction. Traversal,
duplicates, staging collisions, missing/nonregular sources, symlink components,
expected-file hardlink aliases and overlapping roots reject before spawn.

The worker receives copied inputs/support files, prompt references and a fixed
nonsecret environment. Input/expected bytes are not preloaded in prompts.
Expected files and receipts are controller-only, separate from staging.
Trusted synthetic subprocesses are not sandboxed; these tests do not prove
adversarial answer isolation. Native containment remains U5/U7 work.

Synthetic children start in a fresh process group. Timeout/interruption stops and
reaps that owned group, including descendants retaining pipes. Remaining group
members are stopped after normal parent exit. Synthetic fixtures must not detach
into separate sessions; U5 will register detached Claude/Bash groups. Cleanup
never matches process names.

## Independent grading and final submission

| Artifact | Admitted policy | Final evidence |
|---|---|---|
| `file_contents` | `raw_bytes` with normalization only; `normalized_text` with optional heading/block-order/link/source-block flags; `structural` with at least one such flag | Captured first input file |
| `json_envelope` | `structural`, either canonical JSON or exactly one heading/frontmatter/link projection | Strict final JSON |
| `stdout_text` | `raw_bytes` or `normalized_text`, normalization only | Requested UTF-8 final text |
| `stdout_and_file` | File policy above, plus literal `expected_stdout` | Both requested UTF-8 final text and first input file |

Synthetic stdout is the entire explicit final submission. It must contain only
the answer; stderr may carry diagnostics. The controller stores stdout unchanged
as `artifacts/final_submission.bin` for output kinds. It does not strip fences,
guess an answer family, parse provider/tool events, or recover earlier output.
A correct synthetic stderr observation followed by wrong final stdout fails.
U4 must extract untouched terminal receipt text for the same grader API.

Only declared dimensions are compared. Heading-only comparison preserves ordered
rendered levels/text while permitting markup/body differences. Source blocks
preserve fence markers, language, indentation, list/quote markers and line endings.
Block order compares token kinds and nesting. CR/LF define parser line boundaries;
Unicode separators remain within the physical line.

Declared line normalization changes CRLF to LF. Declared trailing-whitespace
normalization removes ASCII space/tab tails, preserving final newlines and
non-ASCII whitespace. H's unrestricted rstrip also discarded those source facts.
Public T2 requires exact inserted content; T10 requires one selected task change.
Their unchanged expected bytes pass packaging checks with the more conservative
normalization. This is not a claim of identical historical grading behavior.

Invalid expected JSON/text rejects before launch. Malformed completed submissions
fail grading. File raw-byte comparisons retain binary capability. Missing required
file capture is unavailable, not semantic failure. The two pure text adapters
never call each other. Synthetic exception/wrong-grade injection proves identical
prompts, grades and stored artifact bytes/hashes for every other family. Changes
to either adapter source still change the whole experiment identity.

## Shared JSON answer shapes

Output instructions take only policy and artifact kind; no expected values.
Only the controller projects frozen legacy expectations. Submitted legacy
envelopes do not become fresh-record authority.

- Headings: ordered `[{"level": 1, "text": "heading"}]`; level must be an
  integer from 1 through 6. No headings is explicitly `[]`.
- Frontmatter: `{"present": true, "format": "Yaml", "value": {...}}`.
  Format and parsed payload are required. Known yaml/toml labels compare without
  case sensitivity because the pinned producers emit Yaml versus yaml; YAML and
  TOML remain distinct. Other strings are not normalized. T21 requires both;
  absent frontmatter requires false/null/null.
- Links: ordered `[{"kind": "link", "destination": "..."}]`, preserving the
  declared kind/destination collection.
- Canonical JSON: preserve the original object/array, fields and meaningful
  array order. Nonempty `json_required_keys` projects every result object,
  requiring each key. `[]` means no projection. Empty-string keys are valid;
  duplicate/non-string configured keys reject.

Strict JSON rejects duplicate keys, nonfinite constants, missing required fields
and booleans as numbers. Decimal parsing preserves exact finite numeric values:
1 and 1.0 are equivalent JSON numbers; 1.0000000000000001 differs from 1.0.
Heading-level integers have their own strict schema. Object-key order is
immaterial; array order remains meaningful. Prose and code fences fail JSON
submission validation.

## Unsupported combinations and evidence limits

Preflight rejects unknown kinds, `multi_file_contents_any`, file-frontmatter
JSON (no independent YAML parser in the locked closure), raw bytes with structural
flags, JSON block/source flags, mixed semantic projections, mixed canonical and
semantic JSON flags, structural policies without dimensions and stdout-text
structural flags. Unused JSON flags on file/text kinds also reject.
These combinations are absent from inspected public policy metadata.
This does not prove sealed core policy compatibility; U8 preflight still owns it.

Public T1/T2/T10/T21 fixtures preserve descriptions, input/expected bytes and
registry contents. Deterministic subprocesses prove packaging; synthetic goldens
prove negative controls. No actual model output was sampled.
Real T14/T23 text-family contracts remain sealed and unvalidated.
Synthetic coverage does not close that gap. Isolated defects in both adapters
can directly invalidate up to 30/360 planned grades; shared or unknown defects
can affect all 360. Either prevents the complete 24-task comparison.

## Receipts and remaining work

Private artifacts use files 0600/directories 0700. New result directories are
exclusive; existing output is never overwritten. Receipts retain stdout/stderr,
final submissions, declared captured files and hashes; result.json is written
last. Timeout/nonzero exit has no semantic comparison. Capture/grader errors
preserve completed execution with unavailable comparison. U4 still owns durable
starts, fsync, recoverable atomic publication, immutable attempts and runtime
record checks.

U3's recipes and build commands are in `cli_eval/tool_reference.md`; its unfrozen
planning template is `cli_eval/experiment.template.json`, not a final study spec.
Source pins are legacy 4d857d2 (0.2.0) and current 6daa2d8 (0.4.1). Builds use
fresh Git exports and locked Cargo dependencies; they never switch/reset this
checkout or substitute installed md. Integration tests build both pins, or
verify preserved receipts under explicit `MDTOOLS_U3_PIN_ROOT` (legacy/current
directories). There is no fourth or forced-full agent condition.

Each condition receives the identical ordinary toolkit and shared task/output
contract. References derive from each executable's schema/help. Source/build,
binary/schema, complete prompt and toolkit bytes bind synthetic identity;
condition executable/schema paths are recorded locators outside content identity.
The unavailable stub exits 1 normally; it is not a Claude permission denial.
Bare macOS mktemp ignores TMPDIR; use an explicit scratch template as documented.
PATH staging alone does not prove cross-condition or answer isolation.

Direct replay runs one current binary on identical frozen synthetic/public bytes
and identical map/query/read argv/stdin, inserting only global --json. It retains
actual stdout/stderr, exits, byte/hash measures and named-tokenizer receipts.
Missing counts remain null/unavailable. Error outputs are separate. These counts
are not provider/billed usage or agent savings. No holdout/mutation replay or
counterfactual reconstructed from untrustworthy agent inputs is performed.

U4 supplies provider
extraction, complete experiment identity and Grade/records. U5 supplies native
containment and Claude shell integration. U6 supplies resume/campaigns/reporting
and CI. U7/U8 need separate paid-run grants. Live isolation remains unverified;
no live action is admitted by U3.

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
`mdtools.cli-eval/1` with backend `synthetic`; they cannot serve as live-study evidence.

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
  serialization and a validated `ExperimentSpec`. Separate task, input/support,
  expected, answer-policy, grader/harness, full prompt, runner/configuration,
  condition, toolkit and dependency identities bind content. Locators are separate.
- `trial_records.py` owns validated execution, grade, usage, event variants and
  immutable attempt keys. Its pure trial disposition selects by ordinal and
  permits at most one explicit transient infrastructure retry. Permission faults
  forbid grading, retries and subsequent admission.
- `command_policy.py` retains P's complete ordinary toolkit plus jq and env, exact
  planned condition names and explicit trusted synthetic argv admission.
  U3 decodes each pinned producer's schema, resolves toolkit targets, and stages
  the exact binary/stub. It also renders and verifies the native Bash boundary.
- `claude_shell.py` is the standalone standard-library launcher. It records
  process-start identities before entering a separate session and the native
  sandbox. Controller cleanup owns those sessions, including job-control groups.

No Pi/OAI/multifile imports, real-provider launch branches, competing correctness
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

Private artifacts use files 0600/directories 0700. `started.json` persists before
spawn; complete raw events flush/fsync as received. `artifact_manifest.json` and
its files/digests precede exclusive atomic publication of `result.json`. The same
finalization is idempotent; a conflicting result is rejected. A crash before
publication leaves an unfinished start. Resume must check its flushed denial trace.

Fresh records use `mdtools.cli-eval/1`. Execution and grade are separate: completed
answers receive pass/fail; timeout, interruption, infrastructure and limits receive
not-run. Capture/grader errors preserve completed execution with unavailable grade.
No compatibility alias or historical result can enter this schema.

The pinned event decoder consumes tool calls/results and one cumulative terminal
receipt. IDs are opaque strings scoped to an attempt. Only untouched terminal
result text reaches the grader. Required terminal permission_denials must be a
valid array; any denial or structured system.permission_denied blocks grading
and admission synchronously. Ordinary tool errors do not infer this fault.
Known quantities from invalid receipts remain partial. Input/output/cache-read/
cache-creation tokens, estimated USD, elapsed seconds and observed tool text bytes
are separate nullable measurements; unknown is never zero.

Portable tests use declared synthetic shapes and subprocesses. The local genuine
CLI fixture check has explicit selection, with no synthetic fallback:

```sh
MDTOOLS_U4_CAPTURED_ROOT=/absolute/private/runner-pins/claude-2.1.272 \
  python -m pytest -q --override-ini 'python_functions=local_test_*' bench/test_harness_json.py
```

Those envelopes came from the actual CLI, but endpoint content and usage were
synthetic. They prove only exercised event shapes. Real-provider values and
unobserved authentication/transport/limit/cache variants remain unverified.

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

U6 supplies campaign-owned identity/schedules, resume, reporting and CI. A single
synthetic attempt's specification is not a live grant or a finished campaign.
Canaries and the first study prefix have zero retries. U7/U8 need separate paid
grants; real-provider compatibility and holdout contracts remain unverified.

## Native offline integration

`LocalClaudeRunner` admits only the preserved Claude 2.1.272 binary and an explicit
IPv4 loopback scripted endpoint. It sends a synthetic API key from a cleared
environment and uses a private empty CLI config. It never copies user credentials
or repurposes HOME. The default command still runs only the synthetic exercise.
There is no paid-provider entry point or inherited run grant.

The actual CLI selects `bash-eval-launcher`; each invocation records its identity
before executing fixed Bash with startup files suppressed. One default-deny macOS
profile permits only fixtures/scratch/snapshot paths and the shared toolkit plus
the selected md/stub. It denies outside reads/writes, network, alternative
executables and signals to other processes. Bash scripts are not security-parsed.
Registration of the observed CLI task-shell form is an additional conformance
check, not the security boundary.

Parent and child use the same staged PATH because the CLI writes its parent PATH
into a sourced shell snapshot. Native tests exercise both md producers through
that real snapshot path; PATH agreement itself is not the isolation mechanism.

The ordinary toolkit adds `env` in every condition because pinned CLI initialization
calls it. This is a recorded capability addition, not historical score parity.
Apple's `dyld-support.sb` supplies process bootstrap rules. The exact
`hw.pagesize_compat` read is required for Rust stack-guard initialization. OS,
system-profile, launcher, rendered profile/config, toolkit bytes and modes are
recorded and checked. The experiment's profile digest names the stable template;
each attempt separately retains the rendered profile and its exact digest.

Closing shell registration precedes cleanup. Only the owned Claude session and
registered Bash sessions are stopped. Unproven cleanup raises and leaves the
private workspace and unfinished start intact; it cannot finalize a success.
Native workspaces are retained with the attempt, even after successful cleanup.

Native tests run on macOS; portable CI cannot establish this OS boundary. Run
actual CLI integration separately with explicit pins:

```sh
MDTOOLS_U3_PIN_ROOT=/absolute/private/u3-pins \
MDTOOLS_U5_RUNNER=/absolute/private/claude-2.1.272/claude \
  python -m pytest -q --override-ini 'python_functions=local_test_*' bench/test_claude_containment.py
```

These tests use real CLI envelopes but scripted model text and usage. They check
Sonnet 5/adaptive/high and Haiku 4.5/thinking-disabled/no-effort request settings,
permission denial, selected binaries, forbidden paths, and timeout cleanup.
Early permission denial stops the CLI before its terminal usage receipt; those
quantities remain unknown. Offline passing results do not prove provider values,
actual model quality, user-auth compatibility, or enforcement of live run limits.
The local one-turn control establishes the CLI's turn-limit receipt/exit behavior;
it does not establish provider billing limits. Valid operational receipts retain
their class when the CLI exits nonzero.

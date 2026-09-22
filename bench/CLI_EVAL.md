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
dependency remains. Corpus files are retained except for the corrected public
section-insertion expectation described below.

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
| `json_envelope` | `structural`, either canonical JSON or exactly one heading/frontmatter/link projection | Final JSON, raw or inside one complete JSON fence |
| `stdout_text` | `raw_bytes` or `normalized_text`, normalization only | Requested UTF-8 final text |
| `stdout_and_file` | File policy above, plus literal `expected_stdout` | Both requested UTF-8 final text and first input file |

Synthetic stdout is the entire explicit final submission. It must contain only
the answer; stderr may carry diagnostics. The controller stores stdout unchanged
as `artifacts/final_submission.bin` for output kinds. The JSON grader accepts raw
JSON or one complete three-backtick fence with an optional lowercase `json`
label. Only JSON whitespace may surround the answer. It never searches prose,
selects an answer from earlier tool output, or changes stored bytes.
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
Their expected bytes pass packaging checks with the more conservative
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
immaterial; array order remains meaningful. Prose, multiple answers and malformed
wrappers fail JSON submission validation. One complete JSON fence is accepted
under `independent-source/2`. Previous receipts retain their original contract
and grades; this rule starts a new experiment. Each authoritative grade has a
hashed `submission_format.json` artifact binding the answer-policy snapshot and
observed raw/fenced/invalid JSON form. Other artifact families have null JSON
form; ungraded executions have no format verdict. Syntax evidence never
determines semantic correctness.

## Unsupported combinations and evidence limits

Preflight rejects unknown kinds, `multi_file_contents_any`, file-frontmatter
JSON (no independent YAML parser in the locked closure), raw bytes with structural
flags, JSON block/source flags, mixed semantic projections, mixed canonical and
semantic JSON flags, structural policies without dimensions and stdout-text
structural flags. Unused JSON flags on file/text kinds also reject.
These combinations are absent from inspected public policy metadata.
This does not prove sealed core policy compatibility; U8 preflight still owns it.

Public T1/T2/T10/T21 fixtures preserve descriptions, input bytes and registry
contents. T2's expected file is the sole historical corpus correction: the
instruction inserts `v2.5` after the pagination paragraph under `v2.0`, while the
historical expectation inserted it before `v2.0`. The regression constructs the
edit from the instruction, accepts that edit, and rejects the historical placement.
Expected bytes for other tasks remain unchanged. New experiments bind the
corrected expected-file digest; prior attempts and their recorded grades are not
rewritten or pooled with corrected experiments. Deterministic subprocesses prove
packaging; synthetic goldens prove negative controls. A stopped public pilot
exposed this expectation defect; it does not establish a tool-performance result.
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
Source pins are legacy 4d857d2 (0.2.0) and current 3853f02 (0.4.1). Builds use
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

U6 implements campaign-owned identity/schedules, resume, reporting and offline CI.
A single attempt's specification is not a live grant or a finished campaign.
Canaries and the first study prefix have zero retries. U7/U8 need separate paid
grants; real-provider compatibility and holdout contracts remain unverified.


### Campaign recovery and reports

`freeze_campaign` validates each task/condition through `prepare_agent` before
creating a `CampaignSpec`. The campaign owns the aggregate identity and immutable
schedule; `ExperimentSpec` still owns each task's content. Input/support/expected,
full prompt, answer policy, executable/toolkit, runner/configuration, harness,
grader, limits, retry and statistics changes require a new campaign. Locators
remain distinct. One model/configuration and common statistical settings are
required; models cannot be pooled.

`run_campaign` holds one whole-campaign lock, scans every stored start/result and
flushed denial event, then admits only the next scheduled pending key. It never
re-executes an unfinished start. An explicit transient infrastructure retry gets
a new ordinal and preserves all costs. The six-entry T14/T23 first-repetition
prefix occurs once at the front of the 360-key core schedule and has zero retries.
Any unavailable grade, contract diagnosis, denial, integrity/configuration fault
or incomplete prefix execution holds before the next spawn. Supported wrong
answers remain ordinary failures. Synthetic tasks with these IDs do not validate
the sealed real contracts.

Per-attempt reservations persist before spawn. When a USD grant is present,
unknown cost holds further launches and retains its reservation as unresolved
exposure; measured zero releases that reservation. Every attempt contributes its
known measurements and coverage. Estimated USD, token/cache categories, elapsed
seconds and observed tool bytes stay separate. A missing category withholds its
aggregate claim; no proxy-unit fallback is used.

`report.py`, the campaign's `report.json`, `--report-bundle` and single-attempt
reports all consume the same validated disposition/coverage path. Cross-bundle
duplicates and foreign records reject. Missing keys, unfinished attempts and
configuration faults produce diagnostic reports with no comparisons. Supported-contract
wrong answers can finish an evaluation with exit 0; incomplete evidence
or admission holds return exit 1; invalid report/configuration inputs return exit 2.
Core completeness requires all 360 scheduled selected outcomes and live evidence;
losing an artifact family cannot complete the 24-task study.

The bootstrap preserves H's task-first/trial-second sampling and exact percentile
indices, seed 1729 and 10000 replicates. Reports retain per-task failures, paired
workflow-success contrasts and costs on paired successful trial intersections.
Total usage coverage still includes failed attempts and retries.

```sh
python -I bench/harness.py --offline-campaign --pin-root /absolute/verified/u3-pins \
  --results-dir /absolute/private/synthetic-campaign --stop-after 3
python -I bench/harness.py --offline-campaign --pin-root /absolute/verified/u3-pins \
  --results-dir /absolute/private/synthetic-campaign
python -I bench/report.py /absolute/private/synthetic-campaign
python -I bench/harness.py --report-bundle /absolute/private/synthetic-campaign
```

This command constructs disposable synthetic fixtures and never reads the task
registry. Resume reconstructs identical bytes; it does not reset prior records.
The Linux `cli-evaluation-offline` job selects root Python tests explicitly,
installs hash-locked dependencies and builds the exact producer pins. It runs no
model, receives no model credentials, reads no holdout contents and uploads no
raw traces. macOS native containment and actual local CLI fixture checks remain
separate evidence duties.

#### Held real-provider API

`ClaudeRunner(executable, endpoint, model, effort, thinking_policy, max_turns,
attempt_usd)` requires an explicit endpoint: an IPv4 loopback string is synthetic;
`None` selects the real Anthropic backend. Configuration verification reads only
pinned executable/version data, never user authentication. There is no paid CLI
command, default grant or automatic model choice.

Real `freeze_campaign` accepts one explicit `claude` configuration. Actual
`run_campaign` and `run_agent` require a matching `LiveRunGrant`, the whole-campaign
lock, canonical schedule membership and a durable per-attempt reservation before
auth environment construction or spawn. A real single task cannot bypass the
campaign. Do not construct a grant unless the user has explicitly approved every
scope and cap it carries. No such grant exists in this implementation session.

Every `LiveRunGrant` field is required: campaign identity, exact model/effort/
thinking policy, phase, task IDs, conditions, repetitions, timeout, turn cap,
per-attempt estimated USD cap, campaign estimated USD cap, maximum attempts,
explicit core-contract-risk acknowledgment and prerequisite experiment identity.
The closed phases are `canary` (one explicitly named synthetic task, all three
conditions, repetition 0, exactly three attempts, zero retries), `public_pilot`
(T1/T2/T10, all three conditions, repetition 0, nine trials), and `core_study`
(all 24 tasks, three conditions, five repetitions, 360 trials). Core consent must
explicitly acknowledge unvalidated T14/T23 contracts and the possible loss of the
complete comparison. Retry attempts count toward the explicit attempt cap.

A public pilot requires a separately granted canary bundle with three matching
native successes, complete traces and known estimated cost. A core study requires
the separately granted nine-trial public pilot with complete matching native
receipts and known costs; pilot semantic failures and fully accounted terminal
turn/cost limits remain valid workflow evidence. Limits retain `not_run` grades,
count as unsuccessful workflows, and do not earn retries. Incomplete receipts,
unknown cost, permission faults, and mismatched models cannot qualify. Synthetic
receipts cannot supply either prerequisite. Model/configuration/condition/source
changes invalidate the prerequisite. A failed live canary stops further launches.
Phase budgets remain separate; reports show prerequisite cost coverage without
pooling its grant with the current campaign.

The initial sealed-contract block has a separate requirement: completed, graded
answers must exercise its previously unvalidated adapters. A terminal limit in
that block still holds subsequent launches because the contract proof is missing;
it does not become a semantic failure or disappear from the recorded outcomes.

Before the first real spawn the controller writes private immutable
`authorization-evidence.json`, including the complete admitted grant, its content
digest and prerequisite locator. Resume requires an explicitly supplied matching
grant and rejects changed audit evidence. This file is evidence, never a source
of consent. A changed phase, attempt count, risk acknowledgment or prerequisite
requires an explicit new run; no frozen study silently gains permission. Reports
validate the same artifact and independently reload prerequisite evidence.

Normal parent authentication is preserved only after grant admission; the Bash
child retains its cleared environment and native filesystem boundary. Provider
receipts are estimates, not invoices. CLI budget flags and the serial controller
cannot promise an invoice cap. Real authentication, transport, provider usage,
sealed contracts and paid canaries/pilot/study remain U7/U8 holds. Portable tests
use invented configuration, grant validation and auth/spawn spies; they never
invoke real authentication or a real provider.
## Native offline integration

`ClaudeRunner` verifies the preserved Claude 2.1.272 binary and source/copy
provenance. With an explicit IPv4 loopback endpoint it sends a synthetic API key
from a cleared environment and uses a private empty CLI config. That mode never
inspects inherited credentials. The default command remains synthetic; the
real-provider API requires the separate grant described above.

The actual CLI selects `bash-eval-launcher`; each invocation records its identity
before executing fixed Bash with startup files suppressed. One default-deny macOS
profile permits only fixtures/scratch/snapshot paths and the shared toolkit plus
the selected md/stub. It denies outside reads/writes, network, alternative
executables and signals to other processes. Bash scripts are not security-parsed.
Registration of the observed CLI task-shell form is an additional conformance
check, not the security boundary.

A permission-conforming completed response with no tool calls is graded normally,
including against unchanged files. The CLI initializes its shell lazily, so only
responses that call Bash require the shell-startup probe. Every actual Bash call
still requires matching launcher evidence; no tool use does not imply a pass.

Parent and child use the same staged PATH because the CLI writes its parent PATH
into a sourced shell snapshot. Native tests exercise both md producers through
that real snapshot path; PATH agreement itself is not the isolation mechanism.
With normal parent authentication, an attempted snapshot write outside the
private shell paths is refused. A scripted-provider control with an empty,
protected parent config proves the CLI still uses the launcher and completes
allowed work without creating that snapshot. This is not a real-login test.

The ordinary toolkit adds `env` in every condition because pinned CLI initialization
calls it. This is a recorded capability addition, not historical score parity.
Apple's `dyld-support.sb` supplies process bootstrap rules. The exact
`hw.pagesize_compat` read is required for Rust stack-guard initialization. OS,
system-profile, launcher, rendered profile/config, toolkit bytes and modes are
recorded and checked. The experiment's profile digest names the stable template;
each attempt separately retains the rendered profile and its exact digest.
Saved source/copy locators come from the preserved runner's sibling `pin.json`;
the updater-owned source is not reopened and may be absent.

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

## Configured preparation and execution

Copy `cli_eval/experiment.template.json` and set its absolute paths. The config
contains task IDs, producer pins, runner settings and limits, not answers or
credentials. Public preparation supports T1, T2, T10, T21 and the separate
synthetic `cli-canary`. Core preparation requires the complete T1–T24 set,
five repetitions, and separately supplied controller-read permission.
Undeclared scopes fail before task content is read.

```sh
python -m bench.harness --prepare-config /absolute/config.json
python -m bench.harness --run-config /absolute/config.json \
  --grant /absolute/explicit-grant.json --prerequisite-bundle /absolute/canary
```

Preparation packages only the declared inputs and expected files, then freezes
`proposal.json`. It does not contact a model or create a grant. Supplying
`--prerequisite-bundle` during preparation checks earlier evidence without
granting launch permission. Run mode regenerates the proposal identity and calls
the existing campaign controller. Resume requires the same config and an explicit
matching grant; saved authorization evidence is audit data, never fresh consent.
The maintained live route checks Max subscription authentication. Confirm extra
usage is off separately before granting a quota-only experiment.

For local tests, supply an explicit synthetic argv with all Claude fields null,
or use the pinned Claude executable with an explicit loopback scripted endpoint.
Neither mode accepts live grants. The `cli-canary` task asks for a fixed file
mutation and uses the same three tool conditions as public trials.

`guidance` has two values. `full_help` remains the default and reproduces the
previous prompt bytes. `discovery` retains the task, strict answer format, file
references and ordinary toolkit but supplies only help-discovery instructions for
md. The no-md prompt is identical in both profiles. Prompt hashes bind this
choice without changing historical experiment or attempt record formats.
Smaller initial prompts are not evidence of better model outcomes.

### Core preparation permission

Both `--prepare-config` and `--run-config` require `--preparation-consent FILE`
for the core corpus. This closed `CorePreparationConsent` JSON records the exact
approval quotation, action `prepare_core_corpus`, schema
`mdtools.core-preparation-consent/1`, absolute source root, full source commit,
Git object IDs for `bench/tasks`, `bench/inputs`, `bench/expected`, `bench/holdout`,
all 24 task IDs, explicit private output roots, and acknowledgment of prior
exposure. Field names and validation are defined in `bench/manifest.py`.
Consent must be explicitly granted; neither a config nor a saved audit file
supplies it. No template manufactures approval.

Consent is checked before core task projection or source reads. Every input,
support and expected file must be inside the consented corpus and match its
committed blob. Changed corpus bytes, unsafe paths and mismatched destinations
reject; parser failures do not print task or answer contents. A real core
configuration also requires its matching model's completed public pilot before
preparation. Synthetic tests use a disposable committed corpus, not real sealed
tasks.

Preparation writes a private immutable consent audit and a frozen 360-trial
proposal. It does not contact a provider. Launch separately requires an
identity-bound `LiveRunGrant` and matching pilot; the original consent must be
supplied again on resume because preparation rechecks source content. Public
and prompt-comparison entry points cannot acquire core access from this flag.
Reports reload already captured receipts without opening the task corpus.

## Paired prompt diagnostic

Use the same config template with exactly T1/T2/T10, one repetition and zero
retries. The comparison command creates two ordinary campaigns under `full_help`
and `discovery`, with identical non-prompt settings. It freezes 18 trials as nine
adjacent pairs. A seeded balanced order determines which profile runs first.

```sh
python -m bench.harness --prepare-comparison /absolute/config.json
python -m bench.harness --run-comparison /absolute/config.json \
  --grant /absolute/two-profile-grants.json --prerequisite-bundle /absolute/canary
python -m bench.harness --report-comparison /absolute/comparison-root
```

The grant file is a closed object keyed by `full_help` and `discovery`, each
containing a separately approved `LiveRunGrant` with phase `prompt_comparison`.
Preparation never generates this file. Both grants must match their campaign and
the same successful candidate-source three-condition canary before public work
starts. A comparison lock serializes dispatch; the existing campaign locks and
admission checks still own each launch. Resume derives progress from receipts,
including a stop between the two halves of a pair. `--stop-after` is an explicit
operator interruption, not a score-based stopping rule.

The independent report preserves each campaign identity. It reports all nine
planned pairs, success differences, discordant outcomes, per-task/condition
counts, four separate token categories and their sum, tool bytes/calls/errors,
elapsed time and nominal USD equivalents. Failed and limited trials stay in the
denominator. Successful-intersection costs have their own denominator. Missing or
invalid evidence withholds the full comparison while retaining available costs.
Three public tasks cannot establish general superiority; the report does not
adopt discovery, tune the prompt or authorize additional trials.

Each live run requires a separate quota-only grant and confirmation that extra
usage is off. No live calls are needed for the maintained-command tests: their
real Claude CLI talks only to a scripted loopback endpoint.

### Haiku diagnostic result — 2026-09-20

This small run is inconclusive. Discovery reduced aggregate token use but did
not establish a better quality/cost trade-off. Full-help remains the default.

The approved run used `claude-haiku-4-5-20251001`, thinking disabled, on source
revision `884960bba3e5ee77dfe989aeb08b5ba29dd76c8d`. Three canaries passed before
the 18 trials on public T1/T2/T10. Each profile had one trial per task and tool
condition. Order was balanced within each condition with seed 1729. No retries,
prompt tuning, grader changes, or additional trials occurred.

| Measurement, all nine trials per profile | Full-help | Discovery |
|---|---:|---:|
| Workflow passes | 6/9 | 5/9 |
| Uncached input tokens | 241 | 228 |
| Cache-creation tokens | 57,429 | 21,941 |
| Cache-read tokens | 498,560 | 374,596 |
| Output tokens | 7,821 | 5,900 |
| Total tokens, including both cache categories | 564,051 | 402,665 |
| Bash tool calls / errors | 42 / 9 | 40 / 5 |
| Tool-output bytes | 74,602 | 11,788 |
| Sum of trial elapsed seconds | 109.56 | 81.87 |
| Reported USD equivalent, not verified cash spend | $0.16098825 | $0.09461385 |

Discovery used 28.6% fewer total tokens across all trials. On the four pairs
where both profiles passed, it used 270,271 tokens versus 263,913: 2.4% more.
The aggregate reduction therefore does not establish cheaper successful work.

| Tool condition | Full-help passes | Discovery passes |
|---|---:|---:|
| No md | 3/3 | 2/3 |
| Legacy md | 2/3 | 1/3 |
| Current compact md | 1/3 | 2/3 |

Both profiles passed three of their six md-enabled trials. The aggregate
one-pass difference came from the no-md control, whose task prompts were
identical. There were two full-help-only passes, one discovery-only pass, four
shared passes and two shared failures. Three tasks and one repetition cannot
separate a stable treatment effect from this variation. Cache state was observed,
not controlled.

Five T1 answers failed raw-JSON packaging because they included prose or code
fences. Discovery also failed T2 with legacy md on block text. Full-help reached
the 12-turn limit on T2 with current md; that trial stayed unsuccessful and
ungraded. None of these outcomes was removed or repaired after the run.

The regenerated report matched the saved report byte for byte. All 21 attempts
had complete usage, the requested model, Bash-only initialization, and no API-key
source. There were no permission faults or remaining owned processes. Canary
usage was $0.02482345 equivalent; all 21 attempts totaled $0.28042555 equivalent,
below the approved $4.80 ceiling. Actual billing remains unverified.

Comparison identity:
`b3906b1b40bb8ea95e5027c9ac9b8092f4d8032ffcd9b9cc3572b7523ebbb290`.
Canary identity:
`00141f92dd97d52bf8cd20154403b8a6a2d32fef1a244b223b401c0e457a9eae`.
Raw receipts remain private and unchanged. Further trials or default adoption
require a new explicit decision.

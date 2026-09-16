# Offline CLI evaluation recovery

U1 restores one synthetic task path. No provider runner is implemented; the
public CLI is synthetic-only. Its default command creates synthetic fixtures and
runs a trusted Python subprocess that edits one staged file, then captures,
independently compares, and persists the result. It never opens the task registry.

## Install and verify

The inspected interpreter is Python 3.9.6 on macOS. The lock pins exact PyPI
pure-Python wheel hashes for markdown-it-py, mdurl, pytest, and every pytest
dependency on Python 3.9. This is an initial lock of installed versions. The
portable offline subprocess implementation requires POSIX process groups.

```sh
python3 -m venv .venv
.venv/bin/python -m pip install --require-hashes --only-binary=:all: -r bench/requirements.lock
.venv/bin/python -m pytest -q bench/test_harness_run_artifacts.py bench/test_harness_task_split.py bench/test_native_runner.py
.venv/bin/python -I bench/harness.py --offline-exercise
```

`python3 -m bench.harness` and `python3 bench/harness.py` select the same offline
exercise by default. An optional `--results-dir` must name a new controller-owned
directory. The JSON output identifies its absolute artifact location. Synthetic
results use `mdtools.cli-eval.offline/0`; they cannot serve as live-study evidence.
This temporary receipt will be replaced by U4's shared validated attempt records.

## Module owners and recovery sources

- `harness.py` selectively recovers H's task fields, fresh staging/subprocess
  mechanics, final-file capture, and artifact-writing intent. H is
  `c93352002e3855527980dbdba0941c8099143e45`, especially harness lines 1227,
  1398, and 1683. Relative staging is repaired to keep the full path below the
  supplied fixture root, rather than flattening to the last directory.
- `neutral_scorer.py` retains H's independent heading parser primitive and moves
  `StructuralDiffPolicy` here from H's harness. U1's file comparison recovers
  the independent raw/text comparison branch, without treatment diagnostics.
  The heading parser is available and smoke-tested, but structural grading is
  not admitted yet. H's lossy block-text primitive is deliberately omitted.
- `manifest.py` recovers H's SHA256 primitives from `v3_manifest.py`; canonical
  serialization and a synthetic-only `ExperimentSpec` replace its historical
  configuration. Old manifest thresholds, headline checks and quarantine rules
  are omitted. U3/U4 must complete experiment identity before live use.
- `command_policy.py` retains the complete ordinary toolkit from P,
  `c8e081301fe845d1d8b9bd9ac125d1dbf7c86fa7`, equalizes jq for later conditions,
  and gives the three planned conditions exact names. It admits only explicit
  trusted synthetic argv. P's eager inventory load and shell-language guard are
  omitted; U3/U5 own binary schemas and actual containment.
- The three test files adapt H's artifact, task-selection/staging and runner
  admission intent using synthetic packages only. Historical provider imports,
  archived bundles and holdout-opening tests are not recovered.

No Pi/OAI/multifile imports, provider launch branches, dual correctness booleans,
quarantine overrides, automatic task generation, historical report policies, or
saved-run dependencies remain in the restored execution path. The corpus is not
a recovery target and is unchanged.

## U1 task and filesystem contract

`run_agent` receives `BenchTask`, disjoint fixture/expected/controller roots,
explicit synthetic argv and a positive finite deadline. Input and support paths
are canonical relative POSIX paths. The first input is the declared final file;
other inputs/support files are staged with their full relative paths. Reject
traversal, duplicates, file/directory collisions, missing/nonregular objects,
symlink components, expected-file hardlink aliases and overlapping roots before
spawn. Supplied roots must not contain symlink components; callers can use the
OS-resolved temporary-directory path when system temporary paths are aliases.

Only `file_contents` with `raw_bytes` or `normalized_text`, and optional declared
line-ending/trailing-whitespace normalization, is admitted in U1. Structural,
JSON and combined/text output families, or any unused comparison flags, fail
preflight until U2. The grader takes captured bytes and the policy, and accepts
no treatment executable. Earlier stdout cannot replace the final file.
An invalid UTF-8 expected artifact under `normalized_text` fails before spawn;
invalid actual text fails the completed submission. `raw_bytes` accepts binary.

Line-ending normalization changes CRLF to LF. Trailing-whitespace normalization
removes ASCII spaces/tabs at each line tail while preserving final newlines and
non-ASCII whitespace. H's branch additionally used unrestricted `rstrip()` and
discarded final newlines. U1 intentionally does not reproduce that extra loss;
U2 must check the original public policies before admitting real tasks under a
completed grading contract. No claim of unchanged historical grading semantics
or completed real-task compatibility is made here.

The worker gets only copied inputs/support files, prompt references and a fixed
nonsecret environment. Input/expected bytes are not preloaded in the prompt.
Expected sources and receipts are controller-only directories, not worker
inputs. U1's trusted synthetic subprocess is not sandboxed: directory staging
and environment clearing do not establish adversarial answer isolation. Native
containment and actual Claude Bash proof remain U5/U7 obligations.

The synthetic executable starts in a fresh session/process group. Timeout or
interruption stops and reaps that owned group, including ordinary descendants
that inherited pipes. Background group members are stopped after normal parent
completion. A synthetic command must not detach into a separate session; U5 will
register and clean up separately detached Claude/Bash groups. No process-name
matching is used.

## Receipts and remaining work

Receipts contain captured stdout/stderr, final first-input bytes when available,
artifact hashes, a synthetic spec, an offline summary, and `result.json` written
last. Files use mode 0600 and controller/artifact directories mode 0700. Existing
result directories are never overwritten. A timeout/nonzero subprocess exit has
no semantic comparison. Missing or symlinked final capture records a capture
error, rather than a semantic failure. U4 still owns durable starts, flush/fsync,
recoverable atomic finalization, immutable attempt keys and runtime record checks.

U2 completes independent grading/final-submission families. U3 supplies pinned
binary/schema references, executable examples and output replay. U4 supplies the
shared records/events/identity. U5 supplies native containment and Claude shell
integration. U6 supplies resume, campaigns, unified reports/statistics and CI.
U7/U8 require separate paid-run grants; no such grant is implied by U1.

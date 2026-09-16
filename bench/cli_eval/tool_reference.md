# Pinned CLI reference and offline recipes

Agent references come from `tool_reference(pin)`: the selected executable's
schema command metadata and its own command help. This file supplies executable
controller recipes, not another command inventory. Shared task facts and
`answer_instructions` precede that reference identically in every condition.

All conditions receive cat, grep, sed, awk, head, tail, wc, tee, mv, cp, mktemp
and jq. Bash builtins remain available. Use an explicit scratch template with
mktemp (`mktemp "$TMPDIR/example.XXXXXX"`); bare macOS mktemp does not use TMPDIR.
Actual symlink targets and executable bytes must be frozen before containment.
Staging PATH does not prevent access to a host binary; U5/U7 must prove that.

## Build and execute the supplied recipes

The following controller invocation builds from the exact source pins in fresh
Git exports and executes every recipe in `exercise_cli_examples`. It requires
cargo; there is no installed-md fallback. Choose a new private output root.

```python
from pathlib import Path
import shutil
from bench.command_policy import (
    CliCondition, build_pinned_condition, resolve_toolkit, stage_condition,
)
from bench.harness import exercise_cli_examples, ReadRequest, replay_read_pair

repo = Path.cwd()
root = repo / "bench/results/cli-eval/new-offline-examples"
cargo = Path(shutil.which("cargo"))
legacy = build_pinned_condition(repo, CliCondition.LEGACY, root / "legacy", cargo=cargo)
current = build_pinned_condition(repo, CliCondition.CURRENT_COMPACT, root / "current", cargo=cargo)
stub = stage_condition(None, root / "no-md/bin", toolkit=resolve_toolkit())
for pin in (stub, legacy, current):
    exercise_cli_examples(pin, output=root / (pin.condition.value + "-examples"))
request = ReadRequest(
    b"# Heading\n\nBefore\n",
    ("read", 'document with "quotes".md', "--query",
     '{"type":"section","text":"Heading","match_mode":"exact"}'),
)
replay_read_pair(current, request, request, output=root / "read-replay")
```

The no-md recipe invokes the unavailable stub, which exits 1 and reports
unavailability. This is an ordinary command failure, not a Claude permission
denial. Legacy recipes execute schema JSON, normal/full outline, and an exact
section read with the legacy producer's SELECTOR FILE argument order.

Current recipes execute schema, compact map and query, original-content section
and preamble reads, stdin address/query inputs, explicit global JSON reads, then
query/map discovery → full guarded block read → JSON patch preview → in-place
rejected stale-etag commit → commit → rejected stale-revision commit →
original-content reread. Guard/revision values
are taken from actual full reads; no values are invented. Preview and stale
rejection preserve document bytes and count zero committed mutations.

## Direct output replay

`replay_read_pair` accepts frozen controller-supplied synthetic/public bytes and
the identical request twice. Both invocations use the same binary and document
path; only global `--json` is inserted. Only map/query/read are replayed.
Different bytes, argv, stdin, existing JSON flags, or mutation requests reject
before replay starts. Every invocation retains effective argv, stdin hash,
stdout/stderr bytes and hashes, exit status, and opt-in CLI measurement.

The tokenizer is named by the producer receipt (currently o200k_base,
tiktoken-rs/0.12.0). Missing counts are null with an unavailable reason, never
zero. Counts describe that encoding's CLI streams; they are neither provider
usage nor billed tokens. Failed reads are a separate error-output category.
Without a trustworthy pre-read snapshot and exact argv, no agent-call
counterfactual is computed. Read-output equivalence currently supports original
Markdown/source and raw-frontmatter views. Frontmatter-field JSON views are
explicitly unsupported; captured invocations do not imply a successful pair.
No model, holdout, or mutation replay is involved.

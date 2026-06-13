# Enabling the pi-json native arm (md vs native Edit for non-Claude weak models)

Date: 2026-06-13
Status: **implemented** 2026-06-13 (Part A + Part B + tests; 223 bench tests + 16 rust tests green).
The GPT 5.4 mini md-vs-native-Edit cell is now runnable on `pi-json`. See "Implemented" below.
Source: `.inbox/2026-06-13_native-edit-baseline-gap.md` (from _blog) + Provi's correction that pi exposes native read/edit/write
Verified against: HEAD `84f9b9f`; live `pi` install (`@earendil-works/pi-coding-agent`); `bench/runs/full-{haiku,gpt54mini}-2026-06-11`

## Why

The June 11 headline (Haiku & GPT 5.4 mini, both 15/28 `unix` → 27/28 `hybrid`, +43pp) grades `md`
against a **shell-only** baseline. The Sonnet falsification (2026-06-04) grades `md` against
**native Read/Edit/Write** and found no reliable advantage. The decisive unmeasured cell is
**`md` vs native Edit for a weak model** — the "tool benefit ∝ 1/capability" thesis predicts `md`
should help a weak model *even against Edit* (weak models are worst at the exact-byte quoting Edit
needs), the opposite of the Sonnet null. Either outcome sharpens the claim.

- Haiku native cell: runnable today on `claude-cli`, no code change (running as `bench/runs/native-haiku-2026-06-13`).
- GPT 5.4 mini native cell: the inbox note claimed pi-json "cannot expose native Edit". **That premise is false.**

## Falsified premise

`pi --help` → *"pi — AI coding assistant with read, bash, edit, write tools"*; `pi tools` lists
first-class lowercase `read` / `bash` / `edit` / `write`. `build_pi_json_command` (`bench/pi_runner.py`)
**already** takes a `tools` tuple and passes it to `pi --tools <comma-joined>`. pi-json is "Bash-only"
purely because the harness hardcodes `tools=("bash",)` at `harness.py:1621`, and `native_runner_error`
(`harness.py:1259`) hard-blocks every non-`claude-cli` runner from native modes. **Enabling the GPT-mini
native cell is a small harness change, not a new runner.** (`oai-loop` genuinely cannot — `oai_loop.py`
accepts only `{"type":"bash"}`/`{"type":"final"}` actions — so it stays blocked.)

## Integrity verdict: `clean-achievable-with-named-guards`

Two independent passes (one adversarial) converged. Key confirmations:

- **The fail-closed no-md preflight already gates the pi-json path.** `_prepend_workdir_to_path` (1531)
  + `_assert_no_md_reachable` (1535) run **unconditionally, before** the runner dispatch (oai-loop 1565 /
  pi-json 1612) and **outside** the `if executor=="guarded"` block. pi inherits the proven-clean env via
  `pi_env = child_env.copy()` (1624). So pi-json native+md-no-md routes through the existing
  `MD_REAL_MODES` predicate — the CLAUDE.md hard rule is satisfied with **no new mode**.
- **pi's `bash` tool sources `BASH_ENV`** (it spawns `/bin/bash -c` via `dist/utils/shell.js`
  `getShellConfig()`; `/bin/bash` exists on this host). So the guard's `PATH=$BENCH_RESTRICTED_PATH`
  override *does* fire for pi — sound defense-in-depth. We do **not** rely on it as the proof (CLAUDE.md);
  the preflight is the guarantee.

### The one real blind spot (must fix before any pi-json no-md run)

pi's `getShellEnv()` **prepends `getBinDir()` (`$PI_CODING_AGENT_DIR` or `~/.pi/agent`, then `/bin`) to
PATH** at spawn time. `_assert_no_md_reachable` probes `child_env`, which does **not** include that dir.
Today `~/.pi/agent/bin` holds only `fd` (no md), so it's benign — but if a real `md` ever lands there,
**the preflight passes while pi resolves the real md** → a silently contaminated no-md ablation. This is
exactly the "benign-today gap that outlives the code fix" pattern the ./md-bypass family keeps producing.
Reproduced empirically by the verifier: a planted real `md` in a pi-bin dir was invisible to the preflight
env but resolvable under pi's PATH in a guard-absent shell.

**Fix:** when `runner=="pi-json"`, prepend pi's `getBinDir()` to the probe env's PATH before calling
`_assert_no_md_reachable`, so the preflight probes the same PATH pi actually runs with.

## Part A — enablement (makes the GPT-mini pass/fail cell runnable)

1. **`native_runner_error` (`harness.py:1259`)** — gate on an allowlist instead of `!= "claude-cli"`:
   ```python
   NATIVE_CAPABLE_RUNNERS = ("claude-cli", "pi-json")  # runners with first-class file Read/Edit/Write
   ...
   if offending and runner not in NATIVE_CAPABLE_RUNNERS:
       return (f"--mode {offending[0]} requires a runner with native file tools "
               f"(claude-cli or pi-json) — got --runner {runner}. oai-loop drives a single "
               f"Bash action protocol and has no Read/Edit/Write to expose.")
   ```
   Fail-closed: a future runner stays blocked from native* until explicitly added.

2. **Per-mode tools at the pi-json call site (`harness.py:1621`)** — mirror the claude-cli toolset
   logic at 1232:
   ```python
   PI_NATIVE_TOOLS = ("read", "bash", "edit", "write")
   PI_BASH_ONLY_TOOLS = ("bash",)
   def _pi_tools_for_mode(mode):
       return PI_NATIVE_TOOLS if mode in NATIVE_MODES else PI_BASH_ONLY_TOOLS
   # call site: tools=_pi_tools_for_mode(mode),
   ```
   All three native modes get the **same** tuple — the native / native+md / native+md-no-md difference is
   whether `./md` is the real binary or the stub (filesystem-enforced via `MD_REAL_MODES`), never the tool list.

3. **Preflight blind-spot fix** (the load-bearing integrity guard above): augment the probe PATH with pi's
   `getBinDir()` for `runner=="pi-json"`. Resolve the dir exactly as pi does — mirror
   `default_audit_extension_path()`'s `$PI_CODING_AGENT_DIR` / `~/.pi/agent` resolution, then `/bin`.

4. **Tests (`bench/test_native_runner.py`)** — `test_native_on_local_runner_is_rejected` currently *pins*
   pi-json as rejected (line 71-77); invert it to reject **only** oai-loop. Add:
   - `native_runner_error` returns `None` for pi-json on every `NATIVE_MODES` mode, non-`None` for oai-loop.
   - `build_pi_json_command` emits `--tools read,bash,edit,write` for native modes, `--tools bash` otherwise.
   - preflight **raises** `NoMdLeakError` when a real `md` is planted in the pi `getBinDir` (proves guard #3).
   - preflight passes when only the stub is reachable across pi's full PATH (workdir + pi-bin + system).
   - preflight runs under `--executor legacy` for pi-json.

After Part A, the GPT-mini md-vs-native-Edit **correctness** cell is runnable. `--mode` is
`action="store"` (single-valued), so loop over the three modes — a single invocation with
repeated `--mode` flags would run only the last:
```sh
for MODE in native native+md native+md-no-md; do
  python3 bench/harness.py --run --runner pi-json --model openai-codex/gpt-5.4-mini --thinking minimal \
    --mode "$MODE" --tasks-path bench/tasks/tasks.json --md-binary target/release/md \
    --results-dir "bench/runs/native-gpt54mini-<date>-$MODE"
done
```

## Part B — instrumentation parity (needed for the note's adoption-split / honesty checkpoint)

pi's `edit`/`write` are built-in tools that **bypass the guarded bash shell**, and
`summarize_pi_audit_events` (`pi_audit_adapter.py:78`) classifies **only** `tool_name == "bash"` into
`call_sequence`. Consequence for a pi native+md run that edits via pi's native tools (the whole point of
the arm): **`mutations = 0`, `requeried = False`**, and the md-adoption split (tool_mix-based,
`report.py:590`) misses pi's native-edit choices. This is the inverse of the claude-cli native arm, which
maps `Edit`/`Write` tool_use → `transcript_mutations` (`harness.py:1695`) and `Read` → query.

- **Pass/fail correctness measures fine without Part B** — it's scored from file/stdout content.
- **The note's "report the adoption split (md_probe_count)" ask needs Part B.** Note `md_probe_count` is
  structurally 0 on native+md for *every* runner (it's the real binary, not the stub — it's the no-md
  leak-gate, not an adoption metric); the adoption signal is `tool_mix`. Because pi's bash sources
  `BASH_ENV`, md-*via-bash* is captured in tool_mix already; what's missing is pi's native edit/write/read.

**Fix:** in `summarize_pi_audit_events`, add `elif tool_name in ("edit", "write")` → append `"mutation"`
and `elif tool_name == "read"` → append `"query"` to `call_sequence`, mirroring the claude-cli transcript
logic. Then `mutations`, `requeried`, and the native-edit-vs-md adoption split populate for the pi arm.

Without Part B, the note's worry partially stands: an agent that satisfies a task with pi `edit`/`write`
+ grep-in-bash shows low md adoption AND `mutations=0`, conflating real native-edit success with md-skip
variance. Report Part-A-only runs as **correctness-only**; do not cite their adoption/requery numbers.

## Sequencing

1. Haiku native cell — running now (no code).
2. Part A + the blind-spot fix + tests — small, unblocks the GPT-mini correctness cell.
3. Part B — before publishing any pi-native adoption-split / requery numbers.
4. Re-measure: any pi-json data collected before guard #3 must be re-run (contaminated-but-looks-clean is
   the failure mode); a re-measure is part of the fix, not optional.

## Implemented (2026-06-13)

All of Part A + Part B landed; 223 bench tests + 16 rust tests green.

- **`bench/harness.py`** — `native_runner_error` now gates on `NATIVE_CAPABLE_RUNNERS = ("claude-cli",
  "pi-json")` (oai-loop still rejected, message updated). Added `PI_NATIVE_TOOLS`/`PI_BASH_ONLY_TOOLS` +
  `_pi_tools_for_mode(mode)` and threaded it into the `build_pi_json_command` call (native modes →
  `read,bash,edit,write`, else `bash`). Added `_pi_agent_bin_dir()`; `_assert_no_md_reachable` gained
  `extra_path_dirs`, and the call site prepends pi's bin dir to the probe PATH for `runner=="pi-json"`
  (the blind-spot fix). Folded `audit_counters.native_tool_mix` into `tool_mix`.
- **`bench/pi_audit_adapter.py`** — `summarize_pi_audit_events` now maps `edit`/`write` → `mutation` and
  `read` → `query`, tracks `native_tool_mix`, and selects `call_sequence` over the bash-only
  `guard_sequence` whenever native tools were used (so native mutations/requery aren't dropped). Bash-only
  runs keep the guard-preferred behavior — no regression.
- **Tests** — `test_native_runner.py`: inverted the rejection test (only oai-loop rejected),
  `test_native_on_pi_json_is_allowed`, and `PiNativeArmTests` (tools-per-mode + the preflight blind-spot:
  passes blind today, raises once pi's bin dir is probed). `test_pi_audit.py`: `PiNativeToolMutationTests`
  (edit/write→mutation, read→query, native-mutation-not-dropped-with-nonempty-guard, bash-only no-regression).

Not changed (deliberately): `NATIVE_MD_DOCS` prose still says "Read/Edit/Write" — it's descriptive, and the
native+md vs native+md-no-md ablation requires byte-identical prompts (shared with the claude-cli arm). pi
maps that prose to its lowercase tools fine.

Run the GPT-mini cell (one mode per invocation — `--mode` does not accept repeats, it's `action="store"`):
```sh
for MODE in native native+md native+md-no-md; do
  python3 bench/harness.py --run --runner pi-json --model openai-codex/gpt-5.4-mini --thinking minimal \
    --mode "$MODE" --tasks-path bench/tasks/tasks.json --md-binary target/release/md \
    --results-dir "bench/runs/native-gpt54mini-<date>-$MODE"
done
```

## Results (2026-06-13, 28 tasks, one run per task/mode)

**The thesis held at the floor for BOTH weak models — md beats native Edit, ablation-clean.**

| Model | Runner | `native` | `native+md` | `native+md-no-md` | md lift vs native Edit | md adoption (native+md) |
|-------|--------|----------|-------------|-------------------|------------------------|-------------------------|
| Haiku 4.5 | claude-cli | 19/28 (68%) | 27/28 (96%) | 19/28 (68%) | **+28pp** | — |
| GPT 5.4 mini | pi-json | 15/28 (54%) | 27/28 (96%) | 15/28 (54%) | **+42pp** | **74%** of tool-calls |

- **Ablation-clean for both:** `native+md-no-md` == `native` (68%/68%, 54%/54%) → the lift is the tool, not the md-advertising prompt. (In native+md-no-md the stub answered "md unavailable" 27/28× — the agent tried md, fell back to native Edit, scored == native.)
- **Inverse of the Sonnet 4.6 null** (2026-06-04): md helps weak agents even against a native editor; strong agents already see the structure. Confirms tool-benefit ∝ 1/capability across the full axis.
- **Efficiency signal:** Haiku native+md used fewer mutations (0.8) than native / native+md-no-md (1.5 / 1.6) — md does structural edits in one shot where native Edit takes several.
- **pi-json native arm, first production run: clean.** No harness/integrity errors. The only failures were task-level: pi's native `edit` tool's non-unique-match on the duplicate-heading tasks (T6, T13: "Found 3 occurrences … must be unique") — exactly the native-Edit weakness md's structural addressing fixes. Part B instrumentation worked: edit→mutation, read→query, native tools in tool_mix → the 74% adoption split above.

Bundles: `bench/runs/native-haiku-2026-06-13` (native+md-no-md), `-native`, `-native+md`;
`bench/runs/native-gpt54mini-2026-06-13-{native,native+md,native+md-no-md}`.

Note: the inbox note's repro command used three `--mode` flags expecting all to run; `--mode` is
`action="store"` so only the last ran. Loop over modes (as above). Flagged back to _blog.

# Decision memo — response to the editor-bench verb-adoption signal

- **Date:** 2026-05-28
- **Trigger:** `.inbox/2026-05-28-verb-adoption-signal-from-editor-bench.md` (editor-bench relaying for fract-ai)
- **Status:** mdtools-owned roadmap response. The signal is *compass*, not a work-item.
- **Grounding:** each finding validated against the actual codebase + mdtools' own
  evidence (`~/.local/share/md/usage.jsonl`, 1,865 real invocations; `bench/`).
  See the provenance footer.

---

## 1. What editor-bench actually needs (the digest beneath the 5 findings)

fract-ai considered making mdtools its **primary** editing substrate (verb-is-op:
each `md` verb → one ChangeSet op against a Lexical-canonical doc). The gating
question was *"do agents actually invoke the verbs?"* — the one datum only the
bench can produce. The bench answered **no**:

- Frontier CLIs (claude, codex) emit **zero** `md` mutation verbs across 92 cells
  each; they use native Read/Edit/Write.
- Only the weak local model (pi) emits them, at pass-rate ≈ its plain baseline.
- claude *does* use `md blocks`/`md outline` to **frame** native edits (21 cells).

So fract demoted mdtools to an **optional capability layer** (FRAC-176),
parked its adapters (FRAC-177/178), and relayed the data **not as a complaint
but as a compass**: they've built a standing measurement loop
(`analyze-mdtools-adoption.ts`) and want mdtools to evolve toward its *real* edge
so the loop stays a live feedback signal.

**The concern beneath the findings:** *don't keep investing in a mutation-verb
surface — especially section-level verbs — that frontier agents will never adopt.*
The standing loop they propose is: bench measures adoption → feeds mdtools →
mdtools evolves → bench re-measures.

## 2. Independent corroboration from mdtools' own evidence

The signal does **not** rest on editor-bench alone. Our own dogfencing log
(`usage.jsonl`, 1,865 real invocations 2026-05-07→29; 1,567 editor-bench cells +
298 genuine Provi frontier-CLI sessions, zero harness pollution) says the same:

| verb | all-time count | note |
|---|---|---|
| `blocks` | 856 | query dominates |
| `replace-block` | 279 | block-level mutation = essentially all mutation traffic |
| `section` | 181 | query |
| `insert-block` | 148 | |
| `outline` | 117 | |
| `delete-block` | 89 | |
| `block` | 78 | |
| `replace-section` | 16 | **14 are Provi's own `/stroke` edits, only 2 from agents** |
| `delete-section` | 3 | |
| **`set-task`** | **1** | the one batch-mutation "win" CLAUDE.md advertises |
| **`move-section`** | **0** | the largest command in the codebase (710 lines) |

Query verbs are ~70% of all calls. The three "zero-uptake" verbs editor-bench
named (`move-section`, `replace-section`, `set-task`) are independently confirmed
dead in real traffic. **Even on mdtools' own relocation-built bench corpus, the
target model solved "move X under Y" via `delete-section`+`insert-block`, never
`move-section`, until the verb was hand-injected into the system prompt — after
which only weak Qwen models used it.**

## 3. Per-finding verdicts (validated against code)

| # | Finding | Verdict | Core grounding |
|---|---|---|---|
| 1 | Structural ops win (+10); within-block edits lose (−8) | **partly-real** | Real architectural root cause — a List/CodeBlock is *one indivisible top-level block* (`parser.rs:278`), so `replace-block` must splice the whole thing. But the −8 cells are **confounded**: weak-model tool-misselection where native Edit fit better, not a tool bug. `replace-block` does exactly what it advertises. |
| 2 | 3 verbs zero uptake; "agents think in blocks not sections" | **partly-real** | Zero-uptake is real and corroborated. But the gloss is too tidy: block *mutation* verbs also get zero frontier uptake — the real axis is **native-Edit beats every `md` mutation verb for capable agents**. `set-task` (loc opacity) and `move-section` (heading-text + relation-flag specification burden) fail for *different* reasons; no single "re-pitch as block ranges" fixes both. |
| 3 | Inspection is the strongest adoption vector | **real & actionable** | The read surface is what frontier CLIs adopt for free. Highest-ROI gap is a **content fingerprint/etag in read output** (was absent everywhere). `md outline` already ships `block_index` + section byte-spans (`outline.rs:72,76-81`), so that part is overrated; the real gaps are etag, sub-block addressing, and search beyond literal substring. |
| 4 | Frontier vs local split is real; pick path (a) or (b) | **strategic, no code** | mdtools' *own* headline already says this (`CLAUDE.md:76`). The binary is false: the code supports a **third path** — inspection-for-frontier + mutation-for-local. Path (a) "win on identity-preservation" is **unbacked today**: `preserves_non_target_bytes` is a hardcoded `true` (`replace.rs`), not a verified post-condition, and `--expect-etag` was deferred. |
| 5 | Adoption-blocker bugs: stale block-index + `--from` heredoc | **real & fixed this session** | See §5. The heredoc *interpolation* corruption (`$VAR`/backtick) is **not** mdtools-fixable — the shell mangles bytes before `read_content` sees them. The real, mdtools-side, adjacent bugs (stale index; trailing-newline) are fixed. |

### Four confounds the memo keeps honest
1. **Prompt-surfacing:** mdtools' pre-2026-05-05 zero `move-section` conflates "never offered" with "offered-and-declined"; editor-bench's frontier zero (agents saw `--help`) is the stronger signal.
2. **Model strength:** every `move-section` invocation anywhere is a weak Qwen/pi model; Provi's frontier use never invokes it — matches `CLAUDE.md:76`.
3. **Selection/denominator:** mdtools' own bench corpus is ~90% relocation by construction and scores *output state*, so its +50pp gap can't speak to within-block edits, and a relocation PASS does **not** imply `move-section` was used.
4. **Scorer-vs-verb:** reading "relocation wins" as "`move-section` wins" mis-reads the evidence — the moves were solved by delete+insert.

## 4. The decision — positioning + prioritized roadmap

**Headline reframe (adopt):** mdtools' value to agents is two surfaces, for two
audiences:
- **Inspection/query verbs** (`blocks`, `outline`, `section`, `tasks`, `search`,
  `table`) — the **frontier-facing** surface. Adopted for free, ~70% of real
  calls. *This is the highest-ROI investment area; improvements compound across
  every frontier CLI.*
- **Block-level mutations** (`replace-block`, `insert-block`, `delete-block`) —
  the **local/weak-agent** surface, plus correctness-under-restriction (hybrid mode).
- **Section-level structural verbs** (`move-section`, `replace-section`, `set-task`)
  — near-dead in observed agent behavior. *Maintenance, not investment.*

Reject finding #4's binary; adopt the **third path** explicitly.

### Priority ladder

| Pri | Item | Effort | Status |
|---|---|---|---|
| **P0** | Per-block `etag` in read JSON + `--expect-etag` fail-closed on block mutations (finding #5 Bug A) | M | ✅ **shipped this session** |
| **P0** | `replace-block` trailing-newline strip (finding #5 Bug B) | S | ✅ **shipped this session** |
| **P1** | Add a **verb-adoption metric to mdtools' own bench** — `harness.py` already captures `mutations`; surface pass-rate split by *which verb produced the diff*. Lets mdtools natively measure what editor-bench measures (the standing-loop enabler on our side). | M | open |
| **P1** | Fix the **CLAUDE.md task-families over-claim**: annotate `move-section` STRONG / `set-task` batch-win with a model-capability + near-zero-frontier-adoption caveat. | S | open |
| **P1** | Write the **positioning** (inspection-for-frontier + mutation-for-local) into README + design rules. | S | open |
| **P1** | Extend the etag/fingerprint to `outline`/`section`/`search` read output (compounds the P0 win across the adopted surface). | M | open |
| **P2** | `replace-block N..M` contiguous block-range (finding #1) — adjacency win on the proven `section.rs` byte-span pattern. **Honest scope: does NOT fix within-block (single list-item/code-line) edits.** | M | open |
| **P2** | `md search --regex` + `--context N` (finding #3) — grep's blind spot is regex+kind-filter+byte-offsets. | M | open |
| **P2** | `md blocks --preview-len N` / `--full`; optional parent-heading `block_index` per entry (finding #3). | S | open |
| **P2** | **Freeze** `move-section` / subsection-relocation candidate growth (two saturation halts, zero frontier uptake). Treat `move_section.rs` as feature-complete-for-correctness. | S | open |
| **P3 / NOT now** | List-item/code-line content mutations (re-enters native Edit's turf; the −8 is tool-misselection, not capability). | L | won't-do unless a consumer needs it |
| **P3 / NOT now** | Make mutation verbs "beat" native Edit via verified `preserves_non_target_bytes` + global etag (path a). Pursue only with the fract oracle-loop as a concrete consumer. | L | gated on consumer |

## 5. What shipped this session (finding #5)

Red→green TDD; `tests/cli_adoption_fixes.rs` (12 tests), all green; full suite
337/337; bench dual scorers all pass; clippy adds no new warnings.

**Bug A — stale block-index guard (`--expect-etag`):**
- `md blocks`/`md block` JSON now carry a per-block `etag` — FNV-1a content
  fingerprint (dependency-free; deterministic across platforms/toolchains, unlike
  std `DefaultHasher`/SipHash), `output::content_etag()`.
- `replace-block`/`delete-block`/`insert-block --before|--after` accept
  `--expect-etag <hash>`; on mismatch they **fail-closed** (exit 4 `EtagMismatch`)
  with a diagnostic that steers a re-query: *"block N etag mismatch … re-run
  `md blocks --json` for current indices and etags, then retry."* This makes the
  read→mutate→re-query moat safe. `--at-start`/`--at-end` reject `--expect-etag`
  (no anchor, exit 3).
- Aligns with the previously-deferred `CLAUDE.md` "`--expect-etag` on all mutation
  commands" intent. Section/task mutations are the remaining slice (P-future).

**Bug B — `--from` trailing-newline ergonomics:**
- The heredoc *interpolation* complaint (`$VAR`/backtick) is **not** mdtools-fixable
  (shell mangles bytes pre-`read_content`); mitigation is docs (`<<'EOF'`).
- The real, fixable, adjacent bug: block spans **exclude** the trailing newline,
  so the trailing `\n` that `cat <<'EOF' > f`/editors/`echo` always append got
  spliced verbatim → spurious blank line + defeated the no-op check. `replace-block`
  now strips one trailing line-ending to match the convention, **gated on whether
  the original block slice ends with `\n`** (so indented-code blocks — whose spans
  *include* the trailing newline — are not truncated). Round-trips now register
  `NoChange`.

Touched: `output.rs`, `model.rs` (BlockEntry +etag), `blocks.rs`, `cli.rs`,
`errors.rs` (+`EtagMismatch`→Conflict), `commands/replace.rs`. JSON change is
additive (`mdtools.v1` retained).

**Scope honesty:** neither fix flips frontier-model verb adoption (the root
signal). They harden the path the local model actually uses and unblock claude's
inspect-then-edit framing — i.e. adoption-blocker *removal*, not the lever.

## 6. The standing loop — accepted

editor-bench's proposal is accepted as-is: they keep running
`analyze-mdtools-adoption.ts` per panel; mdtools ships on this roadmap; fract
integrates later against a matured surface. **mdtools' half of the loop** is the
P1 "verb-adoption metric in our own bench" item — so we can see adoption movement
between panel runs, not only when editor-bench measures.

No reply note was written to `editor-bench/.inbox/` (out of scope for this pass);
the data offer ("happy to slice further") remains open if a P1/P2 item needs it.

---

### Provenance
- Signal: `.inbox/2026-05-28-verb-adoption-signal-from-editor-bench.md` + source notes
  (`editor-bench/.inbox/2026-05-28-frac175-landed-verb-adoption-request.md`,
  `…/2026-05-28-relay-adoption-signal-to-mdtools.md`,
  `fract-ai/.inbox/2026-05-28-bench-response-per-variant-and-verb-adoption.md`).
- Panel: `phase2_panel_all_postsync_2026_05_26` (6 CLIs × 92 cells); analyzer
  `editor-bench/bench/scripts/analyze-mdtools-adoption.ts` @ `02bf853`.
- Grounding: 6-agent validation workflow against `src/`, `bench/`,
  `~/.local/share/md/usage.jsonl` (1,865 invocations).

---

## Addendum — 2026-05-29: fract fit-check resolved; Thread 2 parked

After this memo, the ChangeSet-emitter idea (mdtools emits fract's op-list via
`md diff --ops`) was sent to fract as a **gated** fit-check and answered
(`mdtools/.inbox/.read/2026-05-28-changeset-emit-fit-check-reply.md`).

**fract returned a clean no.** Their reconciler is Lexical-tree-canonical;
"files are not canonical in fract" is a locked product decision (FRAC-176);
there is no lossless markdown base for a diff to consume, and producing one
would re-introduce the lossy markdown↔Lexical roundtrip FRAC-175 exists to
prevent. The emitter would *relocate* that boundary to apply-time, not remove
it (the mirror of the original Probe C finding).

- **Thread 2 (ChangeSet diff-emitter): PARKED.** Unpark condition: fract makes
  markdown the canonical artifact for some surface (their "file-based-space-
  runtime" vision — undecided, not near). They'll signal if it lands.
- **Salvage, filed as backlog (NOT built this pass; not fract-blocking):**
  - **etag → kind-aware fingerprint** — widen the per-block etag from a bare
    content hash to include block kind (fract's `BlockFingerprint.nodeType`
    slot). Independently fixes a same-content/different-kind etag collision
    found in adversarial review.
  - **`move-block`** (index/fingerprint-addressed, forward verb) — the right-
    granularity fix for `move-section`'s ~0 adoption; maps 1:1 to fract's
    `move_block` if a forward path ever connects.
- **Open editor-bench-side action:** add **verb-adoption + cost-to-success**
  metrics to mdtools' own bench ("bench v2"). The token-cost-on-success axis is
  the keystone — it's the only signal where strong-model correctness saturates,
  and it's mdtools' actual unmeasured pitch ("Opus: efficiency only"). Not yet
  scoped; deferred deliberately.

Replies sent: `fract-ai/.inbox/2026-05-29-changeset-emit-clean-no-accepted.md`
and `editor-bench/.inbox/2026-05-29-mdtools-finding5-shipped-loop-accepted.md`.
Original editor-bench signal + fract reply archived under `mdtools/.inbox/.read/`.

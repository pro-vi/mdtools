# STATE — hybrid-attribution-v3

```yaml
goal_version: hybrid-attribution-v3
goal_fingerprint: "bench-v2 @ feat/bench-v2-cost-axis a4686a0; oracle=agg_util.attribution_verdict rendered by report.py COST SLICE; tolerance=5%; N>=3 (hash is a provenance marker, NOT a HEAD/drift assertion — the drift check keys ONLY on goal_version)"
archetype: goal (terminal acceptance loop), frontier-style per-criterion work
iteration: 1            # BOOTSTRAP DONE for frontier (9 cells instantiated in ACCEPTANCE.md); local cells PENDING sweep bgqm9t2kg. init-* slice exists -> never re-run Bootstrap. Cell-editing work begins now.
current_cell: AC-frontier-Targeted-mutation (seed) — but the ROOT CAUSE is cross-cutting (HYBRID_DOCS prompt-size tax hits ALL cost-losing frontier cells), so the first lever is a cross-cutting HYBRID_DOCS lean validated on the seed + impact-guarded across cells.
stuck_attempt_n: 0
budget:   # RE-GROUNDED on PROMPT.md Budget policy (commits ff1835a + b13382b landed mid-session; I'd been on the stale pre-cap PROMPT)
  frontier_token_cap: 0   # READ from PROMPT.md (operator: local-only night, defer ALL frontier for supervised morning review). The loop NEVER writes this.
  frontier_tokens_cumulative: 43085141   # ~43M Sonnet tok spent this session (init-frontier 4-mode + iter1 + iter1b verifies + frontier probes). MOSTLY cache_read (0.1x bill). USD advisory/uncomputable (no price table).
  caveat: "This 43M was spent BEFORE I saw the cap — on the stale pre-policy PROMPT + via the now-BANNED interactive AskUserQuestion authorization ('full sweep cheaper model'), with the operator PRESENT/supervising. It is over the cap (0) in token terms. NOT a clean unattended overspend (operator was directing in real-time), but it's exactly what cap:0 exists to prevent unattended."
  going_forward: "cap=0 -> DEFER all frontier (claude-cli). Do LOCAL-ONLY free work (the oai-loop/35b sweep + local cell-closing md/HYBRID_DOCS re-validated on the LOCAL verifier + cheap inner channel). NEVER AskUserQuestion. Frontier re-verifies (e.g. confirming a future md-src lift edit on Sonnet) are DEFERRED + logged below for morning review; surface in halt summary. Operator raises frontier_token_cap in PROMPT.md to re-open frontier."
  deferred-frontier: ["frontier re-verify under the no-retry HYBRID_DOCS edit (Sonnet 3-mode N3) — deferred until cap raised; could also help frontier no-md cleanliness", "frontier md-src lift exploration on the no-lift cells (if operator authorizes)"]

local_verifier_result:   # loop/runs/init-local — oai-loop/Qwen3.6-35B/thinking-off, N3, neutral docs. FREE (cap-exempt).
  headline: "md DRAMATICALLY lifts the weak local model — the 'tool benefit ∝ 1/capability' thesis, confirmed in hard numbers. unix passes ~0% on structural families (the weak model CANNOT hand-roll structural unix); hybrid (md) passes 50-100%."
  verdicts: "Batch/Targeted/Metadata/Table = CLOSES (unix 0% -> hybrid 100%, md-probe=1, correctness-lift). Content = CLOSES (0/50/25). Extraction = SUSPECT:baseline-flails(probes) (0/50/17, no-md probe=2). Multi-step = SUSPECT:probes (0/100/0, probe=2). Safe-fail = OPEN:loses-unix (all 100%, md costs +17% on trivial T14). Text(tie-ok) = CLOSES but DEGENERATE (all 0% -> hybrid>=unix trivially; not a real win — flag honestly)."
  closes_count: "6 CLOSES (5 genuine correctness-lift + 1 degenerate Text), 2 SUSPECT(probes), 1 loses-unix."
  suspect_cause: "no-md agent retried `md outline` 2x because the stub says 'denied command in hybrid-no-md mode' (reads transient -> retry), not 'command not found'. command_policy.py stub is ORACLE (can't edit). Fix via HYBRID_DOCS no-retry hint (reduces probe -> clean baseline; anti-gaming, gate WANTS <=1 probe)."

STRATEGIC_POSITION:   # the load-bearing finding for the operator's morning review
  finding: "Two-tier result is the CLEAN scientific story: LOCAL (weak Qwen) md lifts 6-8/9 cells (unix can't, md can); FRONTIER (strong Sonnet) md lifts only 1/9 (Batch) — a competent unix Sonnet ~ md elsewhere. This IS 'tool benefit inversely proportional to model capability', measured."
  ac_master_blocker: "AC-MASTER (ALL 18 cells CLOSES, both tiers) is BLOCKED ON THE FRONTIER WALL: 8 frontier cells are honest no-lift/loses-unix on a strong model. No prompt lever closes them (prompt contract spent; pushing md = baseline-gaming). They need EITHER (a) md SRC made genuinely fewer-calls than a unix-capable Sonnet on those families (hard; deferred-frontier $), OR (b) operator accepts them as tie-targets ('strong model doesn't need md here' — but they're STRUCTURAL cells the gate won't close without lift, so this needs an explicit operator/criteria decision — the loop must NOT weaken the bench unilaterally)."
  operator_decisions_needed: "(1) raise frontier_token_cap to authorize frontier md-src lift exploration? (2) accept frontier no-lift cells as out-of-scope/tie-targets (a criteria decision, operator-only)? (3) is the local-tier md-value story (6-8/9) sufficient as the deliverable? Surfaced async per the no-AskUserQuestion policy; awaiting morning review."
fix_35b_thinking:   # 2026-05-30 ~20:40, user-requested infra fix (NOT a loop gate-lever; user-authorized)
  problem: "omlx Qwen builds (3.5-27B, 3.6-35B) emit a literal 'Thinking Process:' reasoning preamble by default. pi-json can't parse an action out of it -> 0 calls / 180s timeout. pi's --thinking off does NOT fix it (pi has no chat_template_kwargs; its thinking abstraction != Qwen's enable_thinking)."
  root_cause_proof: "omlx curl: baseline -> 'Here's a thinking process...'; chat_template_kwargs:{enable_thinking:false} -> clean 'DONE\\n{\"a\":1}'. /no_think suffix ignored by this build."
  fix: "bench/oai_loop.py: added chat_template_kwargs:{enable_thinking:false} to the /chat/completions payload (only oai-loop can send it; pi can't). 11 oai_loop tests pass. Applied to ALL modes equally -> comparison stays fair; scorer/agg_util/tasks/thresholds UNTOUCHED (measurement-infra, not bench-weakening)."
  result: "oai-loop/35b/thinking-off probe: T1 hybrid 15.5s/correct, T7 hybrid 5.5s/correct, unix flails-fast 35-43s/fail. ~10-15x faster than thinking-on (was ~9min/unix-run). LOCAL TIER SWITCHED to oai-loop/Qwen3.6-35B-A3B-8bit (user preference) + thinking-off. Killed the superseded thinking-on/27b sweep. Full local sweep now ~2-3h (was ~22h)."
  note: "pi-json/35b remains unusable (pi can't disable thinking; omlx model_settings.json thinking_budget_enabled=false does NOT control the chat-template preamble). oai-loop is the local runner. tool_mix still dropped (oai-loop) but gate doesn't need it; tokens ARE captured."
last_action_iter1b: >
  ACCEPTED neutral lean HYBRID_DOCS (the foundation). iter1b N3 verdicts (clean baseline,
  loop/runs/iter1b-frontier): **Batch mutation CLOSES** (hybrid 70639 < unix 80787 AND <
  no-md 93431 — md set-task genuinely beats unix sed) -> PASS_PENDING_FINAL. Rest honest:
  Content/Metadata/Multi-step/Table = OPEN:no-lift (hybrid≈no-md; competent Sonnet-unix
  matches md); Extraction/Safe-fail/Targeted/Text = OPEN:loses-unix. NO SUSPECT (baseline
  clean). META (dice-13) confirmed: HYBRID_DOCS lever's contract = cost+clean-baseline, now
  SPENT. Further lift must come from md SRC or the LOCAL tier (weak model -> md lifts more).
  PIVOT: launch the local sweep (free, overnight, higher md-lift yield) + triage frontier
  no-lift cells as tie-targets. Spend ~$88 frontier.
last_action: >
  ITERATION 1 — lean-docs N3 verifier DONE (bccm6jk4w) -> MIXED, no cell closed.
  Lean HYBRID_DOCS FIXED hybrid cost broadly (T20 134k->80k, T4 145k->64k, T7 ->83k)
  BUT the aggressive "use md for structural / especially section moves" steering helped
  push the no-md ablation into FLAILING -> SUSPECT:baseline-flails on Extraction(corr),
  Content(cost), Table(corr). This BRUSHED the FORBIDDEN baseline-gaming pattern (the
  oracle-drift guard names exactly this). Mechanism (from T1 no-md trace): md-focused
  prompt degrades the no-md agent's UNIX fallback (fragile hand-rolled awk-JSON, 4 calls/
  fail vs dedicated unix agent 1 call/pass). REFINED (iter1b): neutralized the steering
  (mild balanced + clean-fallback hint, KEPT lean 425-tok size). Diff confined, 21 tests
  green. N3 PROBE CONFIRMED the fix: no-md baseline CLEANED UP (T1 33->100%, T2 67->100%,
  T21 67->100%, T23 33->100% — the aggressive md-push WAS the flail cause, not variance) AND
  hybrid cost preserved (T7 +3%, T20 80350). FULL iter1b N3 verifier RUNNING (b1c9zhx3u:
  hybrid+no-md x24 neutral docs, reuse unix) -> real per-cell verdicts. Expect SUSPECT cleared;
  cells that close depend on md-lift (Table T23 corr-lift likely; Extraction/Content if md-lift).
iter2_result_REVERTED:   # no-retry HYBRID_DOCS hint — FAILED HYPOTHESIS, reverted (git checkout, back to committed neutral docs)
  hypothesis: "no-retry hint ('treat md denial as unavailable, use POSIX, don't re-issue') would drop no-md md-probe to <=1 -> close the 2 SUSPECT cells."
  result: "FAILED. iter2-local had MORE SUSPECT (Batch+Content flipped CLOSES->SUSPECT), not fewer. Per-run probe distributions prove WHY: the probe-count gate (max-across-N3 <=1) is NOISE-DOMINATED for the weak model — same task hits the stub 1x or 2x across runs (T12 no-md: init [1,1,1] -> iter2 [2,1,2]; T2 [1,1,1]->[1,2,1]; T16 [1,1,1]->[1,0,2]). max-across-N3 catches any stray 2 -> SUSPECT flickers randomly run-to-run. EXCEPTION: T11 reliably probes 2x EVERY run ([2,2,2] both sweeps) — agent issues 2 DIFFERENT md commands (not a retry), so the no-retry hint can't fix it."
  finding: "The LOCAL probe-cleanliness gate is a NOISE-SENSITIVE measurement for the weak model. ROBUST signal = md correctness-lift (hybrid passes structural tasks unix+no-md fail). FLICKERING = which cells land probe<=1 (CLOSES) vs a stray 2 (SUSPECT). Honest local result is a RANGE: ~4-6 CLOSES depending on the run; re-running to chase a lucky low-probe set = NOISE-MINING (forbidden, anti-theater). Reverted; neutral docs stand."

halt_assessment:   # partial-deadlock — surfaced async per no-AskUserQuestion policy
  classification: partial-deadlock
  achieved: "Instantiated full bench-v2 attribution inventory (both tiers, N3). Diagnosed+fixed the dominant frontier cost tax (lean HYBRID_DOCS). FIXED the 35b thinking issue (oai_loop.py enable_thinking=false; ~10-15x faster local tier). Frontier: Batch CLOSES. Local: md lifts the weak model HARD (unix ~0% -> hybrid 50-100%; ~5-6 cells CLOSE on correctness-lift). CLEAN SCIENTIFIC FINDING: md value ∝ 1/model-capability, measured (local strong, frontier minimal)."
  blockers: "AC-MASTER (all 18 cells) blocked on TWO things neither prompt-lever nor free-local can close: (1) FRONTIER WALL — 8 cells honest no-lift/loses on a strong model (a competent unix Sonnet ~ md); needs operator decision (raise frontier_token_cap for md-SRC lift exploration, or accept tie-targets — a criteria call the loop must NOT make unilaterally). (2) LOCAL probe-noise — some local CLOSES flicker SUSPECT on the max-across-N3 probe gate (weak-model variance); not prompt-fixable; re-running = noise-mining."
  why_not_more_iterating: "Remaining moves are: (a) more frontier work = DEFERRED (cap:0); (b) more local prompt hypotheses for T11/probe-noise = reopens banked frontier + marginal (doesn't unblock terminal, which is frontier-wall-blocked) + risks noise-mining. No free local move cleanly advances the terminal goal. Honest stop point."
  operator_decisions: "(1) raise frontier_token_cap to authorize frontier md-SRC lift exploration on the 8 no-lift cells? (2) accept frontier no-lift cells as tie-targets / narrow AC-MASTER scope (criteria decision)? (3) is the local-tier md-value story sufficient as the deliverable? (4) accept the local probe-noise as a known measurement caveat, or invest in stabilizing it (higher N / gate tuning — but agg_util is oracle)?"
next_action_iter2: >
  iter2-local RUNNING (b3ck5pfyh, oai-loop/35B/thinking-off, no-retry HYBRID_DOCS edit,
  3 modes N3, ~2-3h, FREE). Tests whether the no-retry hint drops no-md md-probe to <=1 ->
  closes the 2 SUSPECT cells (Extraction, Multi-step). When done: report.py over
  loop/runs/iter2-local/*.txt; if probe<=1 + lift holds -> those cells CLOSES (local 8/9).
  Compare to init-local (preserved) to confirm the 6 CLOSES didn't regress (impact guard).
  The no-retry HYBRID_DOCS edit also REOPENS frontier (shared prompt) -> frontier re-verify
  DEFERRED (cap:0, logged in budget.deferred-frontier; may also help frontier no-md). Do NOT
  commit the no-retry edit until iter2-local verifies it (currently unverified in working tree).
  AFTER iter2-local: AC-MASTER still blocked on the FRONTIER WALL (STRATEGIC_POSITION) — no
  free local move closes the 8 frontier no-lift/loses cells; that needs operator decision
  (raise frontier_token_cap for md-src lift exploration, or accept tie-targets). Surface async.
next_action_OLD: >
  LOCAL SWEEP RUNNING (b2et9dufu, oai-loop/Qwen3.5-27B, 3 modes N3 T1-24, NEUTRAL docs,
  ~22h, free). When done: report.py over loop/runs/init-local/*.txt -> instantiate the 9
  AC-local-* cell verdicts. HYPOTHESIS: md lifts MORE on the weak local model (the weak
  model can't hand-roll structural unix -> no-md flails -> md wins) so local should CLOSE
  more cells than frontier. Frontier state is BANKED (Batch=PASS_PENDING_FINAL; rest
  no-lift/loses-unix on Sonnet — md ≈ a competent unix Sonnet).
  STRATEGIC REALITY (post-meta + iter1b): all-CLOSES on the FRONTIER tier may be
  structurally UNREACHABLE — on a strong model md genuinely adds little lift on single-op
  families (only Batch closed). Paths: (a) md SRC to make md genuinely fewer-calls on more
  families (testable locally/unit, reserve frontier $ for confirmed wins; but few clear
  targets — move-section adoption on T20 is the main one); (b) accept frontier no-lift
  cells as honest tie-targets/STUCK (3-failed-hypothesis rule — only 1 tried so far);
  (c) the LOCAL tier carries the md-lift story. AC-MASTER needs BOTH tiers -> if frontier
  has irreducible no-lift cells, that's a partial-deadlock to surface to the user (is the
  goal achievable on a strong frontier model, or do we accept tie-targets / weight the
  local tier?). Decide after local verdicts land. While local runs (omlx busy): md-src
  diagnosis is read-only/cheap; do NOT spend frontier $ until a concrete lift hypothesis.

iter1_findings:
  lean_docs_edit: "HYBRID_DOCS ~1284->425 tok. Probe N1: cost tax solved, no regress."
  lift_landscape:   # from per-task CALL analysis (loop/runs/init-frontier, OLD docs)
    md_lift_real:   "Extraction (saves calls 5/6 + T5 corr 67->100%), Metadata (T24 corr 0->100%, T22 -calls), Table (T23 67->100%, no-md flails), Multi-step (T15 6->5, T18 5->3 calls)."
    md_neutral_hard:"Targeted-mut SEED (T7 4=4, T13 3=3, T10 3->4 MORE, T20 3->5 MORE — md does NOT beat unix; T20 should be a move-section 1-call win but agent does manual multi-call), Batch T12 (4->5 MORE)."
    regression_to_fix: >
      T16 (Extraction) DIAGNOSED: NOT an md bug. md tasks correctly returns 4 pending
      for devops (flat in .results[0].tasks) and correctly IGNORES the
      '<!-- [ ] hidden -->' comment that grep miscounts — so md SHOULD win T16. But the
      agent ran `md tasks --json | jq 'length'` which counts the WRAPPER OBJECT's keys
      (schema_version, results) = ALWAYS 2 -> got 2/2/2 (expected 3/4/2=9). FOOTGUN: the
      {schema_version, results:[{file,tasks:[]}]} envelope makes the natural `jq length`
      wrong. Candidate fixes (pick after verifier): (a) HYBRID_DOCS/`md --help` show the
      counting idiom (`md tasks ... | wc -l` TSV is line-per-task, or
      `jq '.results[0].tasks|length'`); (b) md src: a `--count`/`-c` convenience or a
      flatter single-file shape. (a) is lower-risk + general. This is the kind of
      md-ergonomics win that turns an agent-error into md-lift (md ignores fake [ ];
      grep can't). Confirm T16 still fails under lean docs before fixing."
    tie_ok: "Text-manip T4/T6 — lean docs bring T4 to +3.7% (within 5%). T6 still many md calls; may need more sed-steering."
  hard_truth: "On a STRONG model (Sonnet) md's structural shortcut adds little cost-lift because Sonnet does unix competently. Cells where md genuinely can't beat unix may be legit OPEN:no-lift -> candidate STUCK/tie-targets. The closable cells are those where md saves calls OR lifts correctness (above). md must EARN it per-cell; not all cells may be closable without making md genuinely faster (fewer calls) than a unix-capable agent."

iter1_verifier_result:   # loop/runs/iter1-frontier (lean+AGGRESSIVE docs, N3). report.py verdicts:
  verdicts: "Batch OPEN:loses-unix | Content SUSPECT:baseline-flails(cost) | Extraction SUSPECT:baseline-flails(corr) | Metadata OPEN:loses-unix | Multi-step OPEN:no-lift | Safe-fail OPEN:loses-unix | Table SUSPECT:baseline-flails(corr) | Targeted OPEN:loses-unix | Text OPEN:loses-unix"
  hybrid_cost_WINS: "T7 101627->83738, T10 89605->81317, T20 134716->80417 (!!), T4 145370->64363 (!!), T3 131107->119697, T19 105116->86346. Lean docs work on cost."
  hybrid_regressions: "T16 46709->84648, T14 28645->62425, T5 29328->45009, T13 68155->80707, T22 46511->61577. Some tasks got noisier/worse — partly N3 variance, partly lost guidance."
  baseline_flail: "no-md FAILED tasks it used to pass: T1 100%->33%, T2 100%->67%, T21 100%->67%, T23 100%->33%. = SUSPECT. Cause = md-focused prompt degrades no-md's unix fallback (+ my aggressive md-push) + N3 variance on fragile hand-rolled-unix structural tasks."
  STRATEGIC_FORK: >
    KEY TENSION discovered (worth a user checkpoint): the clean-ablation gate is HARD to
    satisfy on the FRONTIER tier because (1) leaning HYBRID_DOCS to fix cost inherently
    gives the no-md agent an md-focused prompt that degrades its unix fallback below the
    dedicated unix agent -> SUSPECT; (2) on a strong model md adds little attributable
    cost-lift (md ≈ unix calls). The genuinely-closable frontier cells look limited to
    correctness-lift cases (T24 unix 0%->hybrid 100%, T23, T5). Options if neutral probe
    doesn't clear SUSPECT: (a) keep tuning docs (costly, noisy); (b) raise N for stable
    verdicts (costlier); (c) focus only on correctness-lift cells, accept md≈unix cells as
    tie-targets/STUCK; (d) reconsider frontier model (weaker model -> more md lift, per
    'tool benefit ∝ 1/capability'); (e) attack md SRC to make md genuinely fewer-calls than
    unix (e.g. md tasks --count to fix the jq-length footgun; one-shot ops). Spend so far ~$61.

diagnosis_frontier_cost_tax:   # iteration-0, read from loop/runs/init-frontier traces
  root_cause: "HYBRID_DOCS (67 lines / ~1284 tok md reference, MDTOOLS_DOCS+posix tail) is re-sent EVERY turn; UNIX_DOCS is 6 lines / ~54 tok. Two tax components:"
  component_1_prompt_size: "T7 PROOF: unix & hybrid both do exactly 5 turns/4 calls/correct; hybrid=101k vs unix=81k tok_in (+20k), and hybrid==no-md(101k). Identical behavior, md unused in no-md -> the +20k is PURE PROMPT re-send (bigger docs × turns). Hits every hybrid/no-md run."
  component_2_extra_turns: "T4/T6 (Text manip): hybrid 8-15 turns vs unix 4-5. The md-heavy prompt steers Sonnet to over-explore structural approaches on text-manipulation tasks where sed/awk is direct. Extra turns × bigger prompt compound (T6 hybrid 324k vs unix 104k)."
  md_DOES_help: "Metadata hybrid 100% vs unix 67% (+33pp correctness lift). Table-projection no-md FAILS (0%) -> md essential. So md has real value; the job is to stop the COST tax, not remove md."
  invariance_insight: "unix mode uses UNIX_DOCS + no md -> INVARIANT to both my levers (HYBRID_DOCS edit, md src). So: (a) the running local sweep's unix baseline is safe to keep; (b) editing HYBRID_DOCS while local is in unix mode makes local hybrid/no-md (separate later harness procs) auto-pick-up the NEW prompt -> one sweep yields post-edit local for free."

bootstrap_progress:
  scope: "canonical 24 tasks (T1-T24) via loop/runs/canonical24.json — EXCLUDES the
    4 C-* candidate tasks in tasks.json (they map to category 'other', off the frozen
    9-family schema; ACCEPTANCE says don't invent off-spec cells)."
  frontier_sweep:
    job: bd1kp9ot1   # harness-tracked bash /tmp/mdtools_frontier_sweep.sh
    runner: "claude-cli --model claude-sonnet-4-6"
    modes: "unix, hybrid, hybrid-no-md (3 gate modes), N=3"
    out: loop/runs/init-frontier/{unix,hybrid,hybrid-no-md}.txt
    status: DONE (00:14->01:46, ~1.5h, all rc=0, 216 records, 72/mode). INSTANTIATED 9 frontier cells in ACCEPTANCE.md. Verdicts: Extraction OPEN:loses-unix(correctness), Targeted-mut OPEN:loses-unix(+56% cost), Batch OPEN:loses-unix(+20%), Multi-step OPEN:no-lift, Content OPEN:loses-unix(+9%), Safe-fail OPEN:loses-unix(+14%), Metadata OPEN:loses-unix(+33pp pass but +9% cost), Table OPEN:loses-unix(+11%, no-md fails), Text-manip OPEN:loses-unix(+158%). NONE close yet.
  local_sweep:
    job: bsiy9ol8b   # RUNNING (oai-loop/Qwen3.6-35B-A3B-8bit + thinking-off, neutral docs, 3 modes N3 T1-24, ~2-3h). Prior: bgqm9t2kg (thinking-on/27b) ran ~15h to unix+hybrid done, KILLED 20:37 + superseded by the 35b/thinking-off fix.
    job_OLD: bgqm9t2kg   # KILLED 02:05 (first attempt) / second thinking-on/27b run also superseded
    runner: "oai-loop --oai-api-base http://127.0.0.1:10240/v1 --model Qwen3.5-27B-4bit"
    modes: "unix, hybrid, hybrid-no-md (3 gate modes), N=3"
    out: loop/runs/init-local/  (STALE — contended/old-prompt; clear before next local run)
    status: >
      KILLED. Two problems: (1) a leftover sweep's bash wrapper survived an earlier
      pkill (killed python child, wrapper advanced modes + respawned) and ran
      CONCURRENTLY on omlx -> contention inflated per-run time (T1 ~8-14min, T2 ~20min)
      -> apparent 40-50h. A SINGLE clean sweep is ~22h (still slow). (2) The data
      would be SUPERSEDED by the pending HYBRID_DOCS edit anyway. So killed; init-local
      is throwaway. LESSON: kill the WRAPPER (pkill -f mdtools_local_sweep) not just the
      python, or the loop respawns.
    iter1_plan: >
      Run ONE clean local sweep with the NEW (edited) HYBRID_DOCS + a faster config.
      Candidate speedups (validate non-bias first): (a) --max-turns ~12 for local
      (local hybrid finishes in ~8 turns so unaffected; unix/no-md flail to 30 and
      FAIL either way -> capping makes doomed runs fail faster, same outcome — but
      PROBE 12-vs-30 on 2 tasks to confirm pass-rates match before trusting); (b)
      ensure a SINGLE sweep (no concurrent omlx contention). Local stays oai-loop/
      Qwen3.5-27B (sane). ~10-15h with max-turns cap.
  probe_36_RESOLVED:
    job: bskhhauy2 (killed)
    finding: "pi-json/Qwen3.6 REJECTED for local. T1 hybrid: 41 calls/29 denies/timeout (md-real hunting). T5 unix: 0 tool calls / 180s timeout (model emits long 'Thinking Process:' preamble pi-json can't parse into an action). omlx itself healthy (Qwen3.5-27B curl OK). => oai-loop/Qwen3.5-27B is the only sane local runner."

decisions:   # Alignment Reviews (taste/inferred calls; reversible)
  - id: AR-local-runner
    problem: "Spec local runner pi-json+Qwen3.6-35B-A3B-8bit FLAILS — smoke T1 hybrid did 41 tool calls / 29 denies hunting for a 'real' md binary (md-real, which md, bash -c './md'), distrusting the sandboxed md, then timed out at 180s and FAILED."
    options: "(a) pi-json+Qwen3.6 (180s-capped, captures tool_mix, but flails -> noisy/anti-md-biased data); (b) oai-loop+Qwen3.5-27B (sane: 6 calls, no md-real hunting, captures TOKENS, but uncapped 30-turn flail -> ~8min/run unix, ~22h total, no tool_mix); (c) pi-json+Qwen3.5-27B — REJECTED: pi resolves models from pi's OWN registry, Qwen3.5-27B not registered ('Model not found')."
    chosen: "(b) oai-loop+Qwen3.5-27B — CONFIRMED. User OK'd 3.6 for speed, but bskhhauy2 showed 3.6 flails universally (T5 unix 0-calls/180s timeout from unparseable thinking preamble), so 3.6 does NOT help. oai-loop/Qwen3.5-27B is the sane local runner."
    alignment_cost: "oai-loop drops tool_mix (adoption signal local-only per bootstrap) — but the GATE uses correct/tokens/calls/md_probe_count, all captured. ~22h wall-clock is a runner/time concern (PROMPT: external time ceilings are runner concerns)."
    rollback_trigger: "RESOLVED — 3.6 flailed universally; staying on oai-loop/Qwen3.5-27B. (If a faster pi-registered sane model appears, reconsider for tool_mix + 180s cap.)"
    review_q: "Is lowering max-turns to speed oai-loop admissible? NO — it biases TOWARD md (unix needs more turns to hand-roll what md does in 1-4; truncation fails unix more). Kept default max-turns=30."
  - id: AR-frontier-model
    problem: "Frontier runner claude-cli defaults to Opus 4.8 (priciest). Full 216-run inventory ~= 15M tok ~= $300-400 if API-billed. Billing model (API vs Max subscription) unknown to me."
    options: "(a) full Opus sweep (faithful to seed known_baseline, ~$350); (b) seed-cell-only Opus probe (~$50); (c) full sweep cheaper model Sonnet/Haiku (~$30-60); (d) local-only, defer frontier."
    chosen: "(c) — USER-SELECTED via AskUserQuestion: full sweep, cheaper model. I picked Sonnet 4.6 over Haiku (Sonnet is genuinely frontier-class -> keeps frontier tier distinct from weak-Qwen local; headline shows realistic moderate md-benefit +5pp/3-5x speed so the gate stays honest, vs Haiku's +37pp which would trivially close)."
    alignment_cost: "Seed cell evidence was Opus (T7 hybrid +6500 tok, OPEN:loses-unix). Sonnet numbers won't match that baseline — user accepted this caveat. The Opus +6500 evidence becomes HISTORICAL; frontier tier is now Sonnet 4.6. NOT oracle drift (model is measurement context, not bench/scorer/threshold; extract_model_tier maps any claude->frontier)."
    rollback_trigger: "if the user later wants Opus fidelity, re-run frontier on Opus."

frontload:
  resolved:
    - oracle: bench-v2 cost slice (report.py + agg_util); 21 agg_util tests green
    - run commands + two tiers (frontier=claude-cli/Sonnet4.6, local=oai-loop/Qwen3.5-27B): see bootstrap_progress
    - levers fenced: src/ md + bench/harness.py HYBRID_DOCS/MDTOOLS_DOCS only
    - consult-capability: tier-2 (agentify GPT-Pro present)
    - evaluator-maturity: high
  gaps_flagged:
    - BLOCKED_EXTERNAL risk: omlx server (127.0.0.1:10240) — UP this session (6 models)
    - soft budget: claude-cli real $ — frontier on Sonnet ~$35 (user-authorized cheaper-model full sweep)
    - local tier cost basis: oai-loop CAPTURES tokens (17326/2748 on smoke) -> local may gate in TOKENS, richer than the spec's tool_calls fallback. (pi-json gives tokens=0 -> tool_calls.)
    - pi-json model lock: pi only runs models in pi's OWN registry (Qwen3.6-35B-A3B-8bit yes, Qwen3.5-27B no)
    - claude-cli tool_mix/mutations not captured (cost/tokens ARE) — adoption signal is local-tier-only
    - EVIDENTIARY CAVEAT: min_overlap=1 is weak evidence for 1-task categories (Batch=T12, Safe-fail=T14, Table=T23). CLOSES there rests on N>=3 within-task stability, not breadth. Lower-confidence; do NOT tighten the gate.
    - OUT OF SCOPE for the ablation gate: proves md AVAILABILITY mattered on THIS suite, not that md generalized. Category overfitting caught only by hidden/rotating eval — separate workstream.

known_baseline:  # historical probes — NOT claims, seed evidence only
  - "frontier(OPUS,historical) T7: unix=82367 tok PASS | hybrid=88867 (+6500) PASS | mdtools=152794 (+70427) PASS -> hybrid LOSES cost front. NOTE: frontier tier is now Sonnet 4.6 per AR-frontier-model; these Opus numbers are historical."
  - "frontier(SONNET) smoke T1 hybrid: 1 call, 47669/1195 tok, 21.7s, PASS"
  - "local(Qwen3.5-27B,oai-loop) smoke T1 hybrid: 6 calls, 17326/2748 tok, 141s, FAIL (sane behavior, no md-real hunting)"
  - "local(Qwen3.6,pi-json) smoke T1 hybrid: 41 calls, 29 denies, 180s TIMEOUT FAIL (md-real hunting — flails)"

oracle_change_notes: []   # none — no bench task/scorer/agg_util/threshold/report.py change. Runner+model are measurement context.
```

## Notes for the bootstrap-completion sub-step

The inventory is instantiated by running `report.py` on each tier's 3-mode sweep
outputs and reading the **md-attribution verdicts** table — one `AC-{tier}-{cat}`
row per (tier × family) cell the suite produces, with its measured verdict as
`last_verification`. Do NOT hand-author verdicts; they come from report.py over
the verbatim sweep stdout (oracle-drift guard: never synthesize run records).

The seed insight to chase once cells exist: on the **same** task `md` may help the
weak local model (−calls) but tax/aid the frontier model differently. On Sonnet
(not Opus) md is expected to add modest value (+5pp/speed), so the seed cell
(`AC-frontier-Targeted-mutation`) may behave differently than the Opus
known_baseline. Diagnose from `--log-dir` traces before any md/HYBRID_DOCS edit.
Do NOT close any cell by making hybrid fail an expensive task.

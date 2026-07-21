# md positioning after v3 and evidence probes

**Date:** 2026-07-04
**Status:** accepted 2026-07-21
**Method:** bench v3 close-out + Plan B probes U1-field, U2 taxonomy, U3 long-document
regime, and U4 transactional multifile regime; later target-state, non-block,
position-bound, and stateless-observability probe lineage.
**Question:** after the honest v3 ruler and the remaining evidence probes, what may
mdtools claim publicly, does Plan C corpus growth activate, and which target-identity
research remains product-relevant?

## Answer

**Keep the broad v3 lift downgraded; publish a scoped weak-model mechanism claim; park
Plan C for now.**

The broad corpus-average claim did not pass the preregistered v3 headline gate. The
effect is real and ablation-clean, but concentrated: +28.3pp, +27.5pp, and +30.8pp
mean lifts have 95% CI lower bounds of only +10.0pp, +9.2pp, and +10.8pp, below the
frozen +15pp certification floor (`bench/RESULTS.md` Headline Status). The README may
say directional/exploratory broad lift, not certified broad lift.

The public positive claim should be scoped to weak/tool-poor models and structure-reading
failure modes: md reduces wrong-target, quoting/escaping, and format/infra-like
tool-use failures in the measured weak-model cells; incomplete-multistep is mixed
across comparisons (`bench/RESULTS.md` Mechanism Evidence). That claim is mechanism
evidence, not a new headline threshold.

## What Changed Since The June Falsification

The 2026-06-04 decision already falsified a robust frontier edge vs native Edit for
Sonnet 4.6 on duplicate-heading, batch-edit, and large-file candidates. It left two
residual regimes open: >10k-line documents and transactional multifile drift safety.
Plan B closed both:

- **Long documents:** CLOSED. On three 24k-39k-line structurally targeted docs,
  `native+md` showed no correctness lift and no byte-cost proxy advantage for either
  Sonnet or Haiku. Sonnet: native 3/9 vs native+md 1/9. Haiku: native 1/9 vs
  native+md 0/9. md was adopted in the md arms but cost more
  (`probes/longdoc/RESULTS.md`).
- **Transactional multifile drift:** CLOSED. Neither native nor native+md reached pass^3
  reliability under live observed-read -> drift -> target-mutation injection. native+md
  did not demonstrate a reliable behavioral edge on either the strong Pi model or Qwen
  (`probes/multifile/RESULTS.md`).

The residual frontier-regime escape hatches are therefore closed as benchmark claims.
`--expect-etag` remains a valid scripting/API safety feature, but not a demonstrated
agent-behavior advantage.

## v3.1 Estimand

Do not re-register the current broad corpus-average `>= +15pp` claim as the public
headline unless the owner explicitly decides to buy more task breadth.

Recommended v3.1 public shape:

- **Primary prose claim:** for weak/tool-poor models, md provides structural Markdown
  access that can materially improve reliability on structure-reading and task-counting
  workflows.
- **Evidence label:** directional/exploratory broad lift, with exact v3 gate FAIL linked.
- **Mechanism label:** exploratory taxonomy evidence over the v3 bundles.
- **Negative boundary:** no demonstrated frontier edge vs native file tools, including
  the newly tested >10k-line and multifile-drift regimes.

If a future certified number is needed, preregister a family-scoped benchmark before
collecting new data. Do not backfit certification onto the existing v3 corpus.

## Plan C Verdict

**Plan C is parked.**

The data leg that would make Plan C statistically useful is true: task variance dominates
86-99% of uncertainty, so more tasks would buy certainty where more repeats would not.
But the positioning choice no longer needs a broad corpus-average certified claim:

- The broad claim already has honest downgrade wording.
- The mechanism claim is more informative than a single aggregate lift.
- U3 and U4 produced no CANDIDATE family needing corpus expansion.
- The frontier-native edge remains falsified, now with the residual regimes closed.

Plan C should activate only if the owner chooses to pursue a certified broad or
family-scoped benchmark headline as a product/marketing asset. If activated later, it
must be gap-blind, spec-sampled, native-adversary gated, and preregistered before data
collection.

## Target Identity Research Disposition

The target-identity probe line is closed as an active product-research line for now:

- the target-state etag candidates all failed to graduate;
- the same candidate family failed to graduate across non-block surfaces; and
- every tested bounded positional-context candidate was demoted.

Those results support the current re-query plus fail-closed ambiguity contract. They do
not prove that durable target identity is impossible.

The final stateless observability-limit protocol was specified but stopped before
execution. Its closed-tuple hypothesis remains untested: none of the protocol's result
labels was earned, and no production identity architecture is authorized from protocol
authorship.

Reopen target-identity research only when a concrete consumer demonstrates a failure
that re-query and current ambiguity handling cannot resolve. Any stateful follow-up must
first compare authority, lifecycle, corruption, portability, cleanup, and concurrency
costs. The canonical lifecycle and evidence links live in
[`probes/README.md`](../../probes/README.md).

## Product And Follow-Up Disposition

- **MCP read probe:** stays shelved. U1-field already answered the adoption question:
  adoption is salience-limited and reads-only, not overkill. A future MCP probe should
  only test whether in-context tool listing beats global instruction salience.
- **loopgen/braid:** spawn an out-of-repo template patch. This is the largest observed
  missed-win population because loop-state markdown is edited repeatedly and structurally.
- **`replace-section` blank-line bug:** file as a product bug; it affects real sessions,
  not benchmark positioning.
- **`md section --contains` / prefix matching:** file as a product feature; exact-heading
  matching misses decorated/dynamic headings.
- **Table row mutation:** keep as roadmap demand, not as benchmark evidence.
- **Braid library dependency:** one real Option-C/library-consumer data point. It supports
  keeping library/API ergonomics alive, but it does not change the benchmark claim.

## README Wording Constraint

Allowed:

- "v3 shows large directional weak-model lifts with clean no-md ablations, but the frozen
  headline gate failed because the task-level confidence intervals are too wide."
- "Mechanism evidence suggests md mostly helps by reducing structure-reading and
  multistep targeting failures in weak/tool-poor models."
- "Frontier native-tool agents do not show a robust md edge; long-document and
  transactional-drift probes are closed as benchmark claims."
- "No tested stateless target-identity candidate graduated; re-query and fail-closed
  ambiguity remain the current contract."

Not allowed:

- "md has a certified +28-31pp benchmark lift."
- "md beats native Edit on large documents."
- "etag safety is proven as an agent advantage."
- "Durable target identity is impossible."
- "The stateless observability limit was validated."
- "Plan C is active."

## Owner Acceptance

Accepted on 2026-07-21. The public claim remains scoped mechanism prose, Plan C stays
parked, and the target-identity research line stays closed unless the reopening gate
above is met.

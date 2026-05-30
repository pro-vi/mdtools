# Skill Harvest — kick-off must be iteration-agnostic; bootstrap self-gates on STATE

- **target skill:** `/loopgen` (SKILL.md Phase 4 + `primitives/runner-contract.md`;
  deeper follow-up in `templates/bodies/goal-body.md`).
- **observed gap:** loopgen emits `loop/PROMPT.md` but gives **no guidance on the
  kick-off** — the line the operator pastes into the runner. The runner (`/goal`,
  ralph, gnhf, cocc) re-sends the *same* prompt every iteration, yet nothing in
  the skill said the kick-off must be iteration-agnostic, and the `goal` body —
  unlike `story` body's `## Bootstrap mode` — has no self-gated bootstrap, so the
  composed goal loop's one-time inventory-instantiation was left ungated.
- **evidence iteration:** mdtools hybrid-pareto goal loop, authoring session
  2026-05-29. The first drafted kick-off ended with "begin with its
  First-iteration bootstrap." The user caught it: under `/goal` that prompt is
  re-sent each iteration, so iteration 2 would re-run inventory instantiation.
  Fix: (a) add an **inventory gate** to PROMPT.md's iteration-protocol step 1 —
  bootstrap runs only when `STATE.md iteration: 0` / no `loop/runs/init-*` slice,
  then ends the iteration; (b) reduce the kick-off to a stable pointer:
  `/goal Read loop/PROMPT.md and run one iteration of the loop it defines; loop/STATE.md tells you where you are.`
- **proposed rule:** (1) Phase 4 must produce an iteration-agnostic kick-off
  (pointer to `loop/PROMPT.md`, no "first/begin/start-by" language); (2) any
  archetype with iteration-0 setup must self-gate it on durable state, story's
  `## Bootstrap mode` being the canonical shape; (3) before emitting, verify the
  composed PROMPT.md actually has that gate. **(1) and (2) are now patched into
  SKILL.md Phase 4 + runner-contract.md.**
- **why it generalizes:** every runner in the family re-invokes the same prompt;
  this is a property of the runner contract, not of one repo. Any loop with
  one-time setup (goal-inventory, greenfield-preloop, frontier-ledger-seed,
  story-storyboard) hits it. Only `story` body had immunity (its Bootstrap mode);
  the other three bodies are exposed.
- **suggested patch wording (the remaining follow-up — NOT yet applied):** add a
  gated `## Bootstrap (iteration-0 only)` section to `templates/bodies/goal-body.md`
  (and frontier/greenfield bodies) mirroring story-body's enter/exit gate, so
  future goal/frontier/greenfield loops self-gate automatically instead of relying
  on operator diligence at compose time. This is a body-template change affecting
  all future loops of those archetypes — hold for explicit approval.
- **accidental-encouragement risk:** an over-broad "always add a bootstrap gate"
  rule could push operators to invent iteration-0 phases where none is needed
  (a stateless frontier loop has no one-time setup). Mitigation: the gate is
  required ONLY when the prompt has genuine one-time setup; for stateless loops
  the kick-off pointer alone suffices and no bootstrap section is emitted.

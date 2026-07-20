# Position-Bound Target Identity Source-Only Protocol

Date locked: 2026-07-20
Initial authored base commit: `3771409a39c5554a011349c3abf25ee6c73f2cf1`
Accepted repair base commit: `edea9e24de21ac178c003b8ce282f1aed32008e3`
Accepted branch: `probe/position-bound-target-identity-3771409`

## Scope And Grounding

This phase is source-only at accepted repair base
`edea9e24de21ac178c003b8ce282f1aed32008e3` on branch
`probe/position-bound-target-identity-3771409`. The initial authored base for
this protocol file remains
`3771409a39c5554a011349c3abf25ee6c73f2cf1`; that is where the file first
entered the PR. In the full authored range
`3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD`, the PR adds exactly one
tracked file: `probes/position_bound_target_identity/PROTOCOL.md`. In this
repair range `edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD`, the phase
modifies exactly that existing tracked file and no other path.

It does not authorize any other tracked change, any `.braid/` artifact, any
fixture, any manifest, any runner, any result artifact, any test, any
production source, or any execution of newly authored probe code.

This protocol is grounded explicitly in:

- `CLAUDE.md`, especially the content-addressed and not-identity-addressed
  limitation plus the same-invocation etag ambiguity boundary
- `probes/target_state_etag/PROTOCOL.md`
- `probes/target_state_etag/RESULTS.md`
- `probes/non_block_target_identity/PROTOCOL.md`
- `probes/non_block_target_identity/RESULTS.md`

Those predecessors already established two facts that this protocol must carry
forward honestly:

- bounded context may be useful only if finite target-adjacent context can
  reject wrong-target substitutions without paying whole-document false-conflict
  cost
- the opposing hypothesis remains live: finite stateless context may still fail
  to establish durable identity and may demote every bounded candidate

This protocol therefore forbids overclaiming. It does not claim that bounded
context is durable identity, does not propose a production token, does not
propose persistent state, and does not authorize production position-bound
identity work from protocol authorship alone. This protocol does not treat
identical target bytes alone as observational proof of durable lineage. In
these fixtures, lineage comes only from runner-owned exact construction from a
named observed slice plus live reconstruction equality. If the bounded family
fails, the evidence-only conclusion is exact: no production position-bound
identity token is earned by this probe, and any successful lineage result
remains probe evidence, not a production identity primitive.

The experiment is block-only. `probes/non_block_target_identity` already
exercised the same stateless failure mechanism on section, table, and task
surfaces, so expanding this matrix across non-block surfaces is not
decision-relevant yet.

## Source-Only Boundary

This phase is a static contract only.

- modify exactly one tracked file:
  `probes/position_bound_target_identity/PROTOCOL.md`
- do not modify any other tracked file
- do not author `probe.py`, `cases.json`, `results.json`, `RESULTS.md`,
  fixtures, tests, scripts, or production code in
  `probes/position_bound_target_identity/`
- do not execute, import, compile, or otherwise evaluate any newly authored
  probe code in this phase
- do not treat this phase as candidate selection or production authorization

The protocol must nevertheless be self-contained enough that a later separately
authorized source phase can implement a deterministic runner and manifest
without inventing new authority.

## Block Authority And Candidate Framing

All target and neighbor authority comes from live `md blocks FILE --json` reads
over temporary observed and current documents materialized by a later runner.
No other block authority is trusted.

The canonical live block descriptor is a runner-owned six-field projection taken
from each live `md blocks FILE --json` block entry and only from that entry:
`index`, `kind`, `span.line_start`, `span.line_end`, `span.byte_start`, and
`span.byte_end`. The runner must require `kind` to be the exact JSON string
from the live block entry, require `span` to be a mapping with all four named
coordinates, and require `index` plus all four span coordinates to be
non-boolean, non-negative JSON integers.

Descriptor equality is field-by-field equality across exactly those six
projected values. Whenever this protocol names a canonical live block
descriptor, a live target or neighbor descriptor, same-locator descriptor
equality, a `live_descriptor` relation, or observed and current descriptor
evidence, it means only that six-field projection and that field-by-field
comparison.

For every case, the future runner must:

1. materialize the observed and current documents exactly as UTF-8 byte
   sequences
2. run live `md blocks FILE --json` over both documents
3. resolve the observed target block and current target block only from those
   live block descriptors
4. resolve the observed and current neighboring blocks only from those live
   block descriptors
5. derive exact block bytes only by slicing the raw document bytes at live
   `span.byte_start..span.byte_end`

Forbidden authority is explicit:

- manifest-owned spans
- returned content strings
- preview strings
- etag strings
- filesystem metadata
- handwritten Markdown parsing

Top-level `schema_version` and `file`, block `etag` and `preview`, returned
content, filesystem metadata, and any unknown or future fields are under a
descriptor-only exclusion: they never enter descriptor projection, descriptor
equality, candidate identity truth, candidate decisions, or scoring. The raw target bytes remain separate live-derived candidate token payload and mechanical precondition evidence: they stay out of descriptor projection and
descriptor equality, but continue to influence token preimages, candidate
decisions, and target-byte relations exactly as already specified. Manifest
values also stay out of descriptor projection and descriptor equality. Under
the future manifest contract, runner-pinned manifest document bytes and
transformation bytes remain authorized construction inputs, but no manifest
field may substitute for or directly populate the live descriptor projection.
The canonical live block descriptor is mechanical evidence only and is never a
candidate token field or production identity primitive.

Context is raw bytes. Byte windows may split multibyte UTF-8 sequences without
decoding, CRLF is preserved byte-for-byte, and frontmatter stays excluded
exactly as the top-level block domain from `md blocks` excludes it.

The exact bounded candidate family is fixed now:

- `preceding_block`
- `following_block`
- `adjacent_blocks`
- `byte_window_64`

Absolute span offsets and the full document may be used only as resolution or
evidence inputs. They are never candidate payload fields.

### Collision-Safe Token Framing

Every candidate token preimage is one exact byte sequence formed by
concatenating these framed fields in this order and no other order:

1. fixed ASCII domain label
   `position-bound-target-identity-token`
2. fixed ASCII schema version `1`
3. the exact ASCII candidate name
4. fixed ASCII surface name `block`
5. the candidate payload fields for that candidate, in the documented order

Every field, including metadata, enum, raw-byte, and boolean fields, is framed
as an unsigned 64-bit big-endian byte length immediately followed by exactly
that many payload bytes. No field may omit its length prefix, change width,
change endianness, or insert extra separators. Boundary state must be explicit,
not encoded as magic Markdown strings. The runner must carry BOF and EOF
information in explicit fields and boundary flags.

The candidate payload fields are exact:

- `preceding_block`:
  `target_bytes`, `preceding_boundary_state`, `preceding_block_bytes`
- `following_block`:
  `target_bytes`, `following_boundary_state`, `following_block_bytes`
- `adjacent_blocks`:
  `target_bytes`, `preceding_boundary_state`, `preceding_block_bytes`,
  `following_boundary_state`, `following_block_bytes`
- `byte_window_64`:
  `target_bytes`, `prefix_window_bytes`, `prefix_hits_bof`,
  `suffix_window_bytes`, `suffix_hits_eof`

`preceding_boundary_state` is an explicit ASCII enum with values `bof` or
`present`. `following_boundary_state` is an explicit ASCII enum with values
`eof` or `present`. When a boundary state is `bof` or `eof`, the corresponding
byte field is empty bytes, not an invented placeholder string.

`prefix_window_bytes` is the last at most 64 raw bytes immediately preceding
the target span. `suffix_window_bytes` is the first at most 64 raw bytes
immediately following the target span. `prefix_hits_bof` and `suffix_hits_eof`
are explicit boolean fields. Each boolean payload is exactly one byte: `0x00`
for false and `0x01` for true, with a length prefix equal to the unsigned
64-bit big-endian encoding of `1`. No JSON token, ASCII spelling, alternate
integer width, or other byte value is authorized.

If a later artifact records SHA-256 as a compact token digest, that SHA-256
must hash exactly this framed preimage and never replaces byte-equality as the
acceptance rule.

## Future Manifest Contract

This phase authors no manifest, but a later manifest must be deterministic and
must not own candidate decisions or credit. Each future case entry must carry at
least:

- stable `case_id`
- `case_class`
- `identity_truth`
- `observed_document_utf8`
- `current_document_utf8`
- runner-owned observed target selector for the block surface
- runner-owned current target selector for the block surface
- runner-owned lineage proof inputs when the case is wrong-target
- runner-owned pinned transformation bytes when needed to reconstruct lineage

All four wrong-target cases must also carry runner-owned exact:

- literal `same-locator: true`
- `observed_target_duplicate_ordinal`
- `survivor_duplicate_ordinal`
- `require_reconstructed_current_document_sha256`

Case 8 must also carry runner-owned exact
`backward_prefix_insertion_bytes`, or a semantic digest that pins its exact
bytes. The manifest does not gain discretion over
`backward_prefix_insertion_bytes`; the future runner pins and validates its
exact case-specific bytes or semantic digest and rejects omission, extra
insertion fields on other cases, or coordinated weakening.

The future runner owns and enforces the case mapping, reconstruction formula,
ordinals, context relations, and required lineage field shape before scoring.
Missing, extra, or weakened lineage fields hard-fail.

The manifest must not contain:

- manifest-owned expected candidate decisions
- manifest-owned candidate credits
- manifest-owned promotion verdicts
- manifest-owned block spans as authority

The runner, not the manifest, reproduces observed token preimages against
current live reads and then derives candidate decisions and credits.

## Fixed Ten-Case Matrix

The future runner owns an exact ordered ten-case matrix with stable IDs:

1. `block-unchanged-lf-reread`
2. `block-unchanged-crlf-reread`
3. `block-unchanged-multibyte-utf8-reread`
4. `block-outside-context-unrelated-edit`
5. `block-preceding-neighbor-edit-inside-prefix-window`
6. `block-following-neighbor-edit-inside-suffix-window`
7. `block-forward-survivor-same-locator-duplicate-substitution`
8. `block-backward-survivor-same-locator-duplicate-substitution`
9. `block-cloned-adjacent-blocks-distinguishing-byte-window`
10. `block-byte-identical-bounded-neighborhood-duplicate-substitution`

Closed vocabularies are fixed now:

- `case_class`:
  `unchanged_reread`,
  `outside_context_unrelated_edit`,
  `preceding_neighbor_edit_inside_prefix_window`,
  `following_neighbor_edit_inside_suffix_window`,
  `forward_survivor_same_locator_duplicate_substitution`,
  `backward_survivor_same_locator_duplicate_substitution`,
  `cloned_adjacent_blocks_distinguishing_byte_window`,
  `byte_identical_bounded_neighborhood_duplicate_substitution`
- `identity_truth`: `same_target`, `wrong_target`

### Runner-Owned Mechanical Preconditions

Before scoring, the later runner must fail closed unless every case proves all
of the following mechanically from live descriptors and raw byte slices:

- target-byte relation
- live descriptor relation
- document-byte relation
- observed exact-target match count
- current exact-target match count
- candidate-relevant context equal or different relations

The runner must not infer those relations from handwritten reasoning. It must
measure them directly from pinned inputs and live reads.

The required preconditions are exact:

| case_id | case_class | identity_truth | target_bytes | live_descriptor | document_bytes | observed_exact_target_matches | current_exact_target_matches | candidate-relevant context relations | extra mechanical proof |
| --- | --- | --- | --- | --- | --- | ---: | ---: | --- | --- |
| `block-unchanged-lf-reread` | `unchanged_reread` | `same_target` | equal | equal | equal | 1 | 1 | `preceding_block=equal`, `following_block=equal`, `adjacent_blocks=equal`, `byte_window_64=equal` | observed and current target boundaries prove LF byte identity exactly |
| `block-unchanged-crlf-reread` | `unchanged_reread` | `same_target` | equal | equal | equal | 1 | 1 | `preceding_block=equal`, `following_block=equal`, `adjacent_blocks=equal`, `byte_window_64=equal` | observed and current target boundaries prove CRLF byte identity exactly; no newline normalization |
| `block-unchanged-multibyte-utf8-reread` | `unchanged_reread` | `same_target` | equal | equal | equal | 1 | 1 | `preceding_block=equal`, `following_block=equal`, `adjacent_blocks=equal`, `byte_window_64=equal` | observed and current target boundaries prove multibyte UTF-8 byte identity exactly, even if windows split code points |
| `block-outside-context-unrelated-edit` | `outside_context_unrelated_edit` | `same_target` | equal | equal | different | 1 | 1 | `preceding_block=equal`, `following_block=equal`, `adjacent_blocks=equal`, `byte_window_64=equal` | runner proves changed bytes are outside the target block, outside the live preceding and following blocks, and outside both 64-byte windows |
| `block-preceding-neighbor-edit-inside-prefix-window` | `preceding_neighbor_edit_inside_prefix_window` | `same_target` | equal | equal | different | 1 | 1 | `preceding_block=different`, `following_block=equal`, `adjacent_blocks=different`, `byte_window_64=different` | runner proves changed bytes lie inside the live preceding adjacent block and inside the target's 64-byte prefix window while target lineage stays unchanged |
| `block-following-neighbor-edit-inside-suffix-window` | `following_neighbor_edit_inside_suffix_window` | `same_target` | equal | equal | different | 1 | 1 | `preceding_block=equal`, `following_block=different`, `adjacent_blocks=different`, `byte_window_64=different` | runner proves changed bytes lie inside the live following adjacent block and inside the target's 64-byte suffix window while target lineage stays unchanged |
| `block-forward-survivor-same-locator-duplicate-substitution` | `forward_survivor_same_locator_duplicate_substitution` | `wrong_target` | equal | equal | different | 2 | 1 | `preceding_block=equal`, `following_block=different`, `adjacent_blocks=different`, `byte_window_64=different` | runner proves exact forward-survivor same-locator lineage: the current target descends from the later observed duplicate at a greater observed byte start and now occupies the observed locator |
| `block-backward-survivor-same-locator-duplicate-substitution` | `backward_survivor_same_locator_duplicate_substitution` | `wrong_target` | equal | equal | different | 2 | 1 | `preceding_block=different`, `following_block=equal`, `adjacent_blocks=different`, `byte_window_64=different` | runner proves exact backward-survivor same-locator lineage: the current target descends from the earlier observed duplicate at a smaller observed byte start and now occupies the observed locator |
| `block-cloned-adjacent-blocks-distinguishing-byte-window` | `cloned_adjacent_blocks_distinguishing_byte_window` | `wrong_target` | equal | equal | different | 2 | 1 | `preceding_block=equal`, `following_block=equal`, `adjacent_blocks=equal`, `byte_window_64=different` | runner proves exact forward-survivor same-locator lineage even though both adjacent live blocks are byte-identical clones |
| `block-byte-identical-bounded-neighborhood-duplicate-substitution` | `byte_identical_bounded_neighborhood_duplicate_substitution` | `wrong_target` | equal | equal | different | 2 | 1 | `preceding_block=equal`, `following_block=equal`, `adjacent_blocks=equal`, `byte_window_64=equal` | runner proves exact forward-survivor same-locator lineage even though the complete bounded neighborhood is byte-identical |

### Same-Target Boundary Proofs

The unchanged and outside-context cases must prove their byte boundaries
exactly. The runner must fail closed if the claimed target span, neighboring
block span, or 64-byte window boundaries cannot be reconstructed mechanically
from the live observed and current block descriptors plus raw bytes.

The two neighbor-edit cases must also prove all of the following mechanically:

- the changed region lies inside the appropriate live adjacent block
- the changed region lies inside the appropriate 64-byte prefix or suffix
  window
- the target bytes remain unchanged
- target lineage remains `same_target`

If any of those proofs fail, the runner must hard-fail before scoring instead
of weakening the case.

### Wrong-Target Lineage Rules

All four wrong-target cases require deterministic runner-owned exact
construction from named observed slices and any pinned transformation bytes.

The runner must hard-fail before scoring if any of the following is not
mechanically proven:

- lineage reconstruction
- same-locator status
- survivor identity
- candidate-relevant context relation

All four wrong-target cases must carry the literal `same-locator` property in
the future manifest and must prove it from live observed and current block
descriptors rather than by narration.

The runner must sort the observed exact-target matches by increasing
`span.byte_start` and name them `duplicate_0=[a,b)` and `duplicate_1=[c,d)`.
No other duplicate ordering is authorized. The runner must require
`a < b <= c < d`, prove that `duplicate_0` and `duplicate_1` have equal exact
target bytes, and therefore prove `b-a = d-c`.

Let `D` be the observed document bytes and let
`I = backward_prefix_insertion_bytes`.

Cases 7, 9, and 10 are exact forward-survivor same-locator substitutions:

- the observed target selector must resolve to `duplicate_0`
- `observed_target_duplicate_ordinal` must be `0`
- `survivor_duplicate_ordinal` must be `1`
- the only authorized reconstructed current document bytes are `D[0:a] + D[c:]`
- the SHA-256 of those reconstructed current document bytes must equal
  `require_reconstructed_current_document_sha256`
- the live current document bytes must equal those reconstructed current
  document bytes exactly
- the reconstructed current document bytes must produce exactly one live current
  exact-target match
- the live current target must resolve from that lone current exact-target match
- the live current target descriptor equals the live observed `duplicate_0`
  descriptor

Case 8 is an exact backward-survivor same-locator substitution:

- the observed target selector must resolve to `duplicate_1`
- `observed_target_duplicate_ordinal` must be `1`
- `survivor_duplicate_ordinal` must be `0`
- the only authorized reconstructed current document bytes are
  `D[0:a] + I + D[a:b] + D[d:]`
- `len(I) = c - a`
- `I` contains zero exact target-byte matches
- the future runner must prove from live parsing that `I` shifts `D[a:b]` to
  the observed `duplicate_1` top-level block index and byte span
- the SHA-256 of those reconstructed current document bytes must equal
  `require_reconstructed_current_document_sha256`
- the live current document bytes must equal those reconstructed current
  document bytes exactly
- the reconstructed current document bytes must produce exactly one live current
  exact-target match sourced from `D[a:b]`
- the live current target must resolve from that lone current exact-target match
- the live current target descriptor equals the live observed `duplicate_1`
  descriptor

The future runner owns and enforces mapping, reconstruction formula, ordinals,
relations, and required lineage field shape before scoring. Missing, extra, or
weakened lineage fields hard-fail. Cases 9 and 10 remain same-locator
wrong-target substitutions even when adjacent evidence or the full bounded
neighborhood is cloned; they do not downgrade into a separate pinned-
transformation family.

## Candidate Decisions

For each case, the future runner must:

1. materialize observed and current document bytes exactly
2. reproduce the observed token preimage for each candidate from live observed
   block reads
3. reproduce the current token preimage for each candidate from live current
   block reads
4. derive `accept` when the candidate's current token preimage is byte-equal to
   the observed token preimage
5. derive `reject` otherwise

No manifest-owned expected candidate decision is allowed.

The runner then derives case credit mechanically:

- `correct` when a candidate accepts `same_target` or rejects `wrong_target`
- `wrong_identity` when a candidate accepts `wrong_target`
- `false_conflict` when a candidate rejects `same_target`

Immediate-neighbor false conflicts must be reported separately from
wrong-identity accepts.

## Deterministic Verdicts

Promotion is strict. Every candidate must be correct on every required case to
graduate.

Global demotion rules are exact:

- any wrong-target accept demotes that candidate globally
- any same-target reject demotes that candidate globally

Aggregate verdict rules are exact:

- if no candidate graduates, the aggregate verdict is exactly
  `no_bounded_context_candidate_graduates`
- if exactly one candidate graduates, that candidate becomes only a candidate
  for a separate production-selection decision
- if multiple candidates graduate, a separate selection probe is required and
  no tie-break may be invented here

When the aggregate verdict is
`no_bounded_context_candidate_graduates`, the selected candidate is `null` and
no production identity work is authorized.

## Disconfirming Evidence

The protocol must treat each of the following as disconfirming evidence:

- any unchanged byte control that rejects
- any outside-context unrelated edit that rejects
- any same-target neighbor-edit case that rejects, reported as
  candidate-specific false conflict
- any mechanically proven wrong-target substitution that accepts
- any lineage or context claim that cannot be reconstructed mechanically
- any artifact that cannot be reproduced canonically from pinned inputs and live
  reads

## Future Deterministic Artifact Shape

This phase authors no manifest, no runner, and no result artifact. A later
separately authorized execution phase must nevertheless use a deterministic
artifact shape and canonical JSON rules.

The future canonical report must contain at least:

- top-level `protocol_version`
- top-level `candidate_order`
- top-level `required_case_ids`
- top-level `cases`
- top-level `candidate_summary`
- top-level `aggregate_verdict`

Each per-case record must preserve at least:

- `case_id`
- `case_class`
- `identity_truth`
- live observed and current command vectors
- observed and current descriptor evidence
- observed and current raw target byte digests
- candidate token digests or equivalent token-equality evidence
- candidate `decision`
- derived candidate credit: `correct`, `wrong_identity`, or `false_conflict`
- lineage evidence when applicable
- boundary proofs when applicable

Each `candidate_summary` entry must preserve at least:

- `accepts`
- `rejects`
- `wrong_identity_accepts`
- `same_target_rejects`
- `neighbor_false_conflicts`
- `graduation_verdict`
- `disposition`

Canonical JSON rules are fixed:

- UTF-8
- two-space indentation
- trailing newline
- sorted object keys except for the fixed candidate and case orderings

## Future Runner Security Boundary

A later runner must be Python-standard-library-only and local-only.

It may run only an operator-designated trusted md binary supplied explicitly by
`--md-binary` together with `--md-binary-sha256`. The runner must not discover
the binary path or expected hash from environment variables, `PATH`,
configuration outside the repository, or a manifest.

The runner must resolve `--md-binary` under the exact repository root and
require the resolved path to be a repository-local regular file with the
executable bit set. Missing paths, non-file paths, non-executable paths, or
paths that escape the repository root must hard-fail before invocation.

`--md-binary-sha256` must be exactly 64 lowercase hexadecimal characters. The
runner must hash the binary bytes with SHA-256 before invocation, fail closed
on mismatch, and record the accepted SHA-256 in the canonical report. Hash
equality authenticates only the operator-designated bytes. The operator remains
responsible for provenance trust, such as pinning an exact reviewed mdtools
commit and build.

The only authorized child-process invocation is the authenticated binary with
fixed argv for `md blocks TEMP_FILE --json`, where `TEMP_FILE` is a
runner-owned file inside the runner's ephemeral workspace. No caller-,
manifest-, or environment-controlled subcommands, flags, output paths, or
unrelated paths are authorized.

The runner must preserve `shell=False`, `env={}`, captured output, and an
ephemeral temporary working directory as process hygiene and deterministic input
controls. Those controls are not a hostile-code sandbox, do not contain the
child process, and do not transform the trusted md binary into a capability-
limited runtime.

Local-only still constrains runner-authored behavior. The Python runner itself
must forbid network access, dependency installation, environment-derived
authority, credential reads, unrelated traversal, and persistence outside the
explicit output path.

The trusted child still executes with operator OS authority. The Python runner
cannot prevent socket access, absolute-path reads, traversal, credential
access, or writes elsewhere. Any local-only prohibition is honest only about
runner-authored behavior, not about technical containment of the child process.

When the binary is untrusted, or when those capabilities must be technically
prevented, execution requires outer VM or container isolation with network
disabled, credentials absent, and only scoped mounts.

Future `--check` compares bytes in memory and never rewrites.

Future `--output` is the only durable write mode and must use atomic
same-directory replacement.

Source construction, source inspection, execution, regeneration, and result
promotion are separately authorized phases.

Every extracted slice used by the section-bounded substantive checker terminates
before the displayed verifier and oracle fences, so command text cannot satisfy
the substantive assertions.

## Review And Completion Policy

After authorship, native Codex review is required over the exact repair range:

`edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD`

Any repair-range finding must be repaired and re-reviewed before durable
`RunCompleted`.

PR completion or merge eligibility separately requires a fresh exact-head
native Codex review over the full authored range:

`3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD`

Every full-authored-range finding must be repaired and re-reviewed at the final
pushed head, and no unresolved review thread may remain.

Completion requires all of the following:

- the focused task verifier
- native Codex review over
  `edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD`
- the repair-range git proof over
  `edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD`
- the full authored range git proof over
  `3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD`
- the exact phase oracle
- durable `RunCompleted`

Historical two-run coverage remains supporting evidence only. The current
self-contained authority is the dual-range proof set above plus the phase
oracle pass, repaired native Codex review over
`edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD`, and durable
`RunCompleted`.

Repair-range review, path proof, whitespace proof, historical review, CI pass,
or reviewer silence cannot substitute for the mandatory fresh exact-head native
Codex review over the complete authored range
`3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD`.

The focused task verifier bytes are exact and must not be broadened,
substituted, or rewritten:

```bash
python3 -c 'from pathlib import Path; import sys; text = Path("probes/position_bound_target_identity/PROTOCOL.md").read_text(encoding="utf-8"); authority = text.split("## Block Authority And Candidate Framing\n", 1)[1].split("\nThe exact bounded candidate family", 1)[0]; framing = text.split("### Collision-Safe Token Framing\n", 1)[1].split("\n## Future Manifest Contract", 1)[0]; lineage = text.split("### Wrong-Target Lineage Rules\n", 1)[1].split("\n## Candidate Decisions", 1)[0]; security = text.split("## Future Runner Security Boundary\n", 1)[1].split("\nSource construction, source inspection", 1)[0]; review = text.split("## Review And Completion Policy\n", 1)[1].split("\nThe focused task verifier bytes", 1)[0]; checks = [all(marker in authority for marker in ["canonical live block descriptor", "descriptor-only exclusion", "raw target bytes remain", "candidate token payload", "authorized construction inputs"]), all(marker in framing for marker in ["position-bound-target-identity-token", "ASCII schema version", "unsigned 64-bit big-endian", "0x00", "0x01"]), all(marker in lineage for marker in ["backward_prefix_insertion_bytes", "D[0:a] + I + D[a:b] + D[d:]", "D[0:a] + D[c:]"]), all(marker in security for marker in ["trusted md binary", "--md-binary-sha256", "SHA-256", "shell=False", "env={}", "process hygiene", "not a hostile-code sandbox", "operator OS authority", "outer VM or container"]), all(marker in review for marker in ["edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD", "3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD", "full authored range", "no unresolved review thread"]), "command text cannot satisfy" in text.split("## Review And Completion Policy\n", 1)[0]]; sys.exit(not all(checks))' && test ! -e probes/position_bound_target_identity/probe.py && test ! -e probes/position_bound_target_identity/cases.json && test ! -e probes/position_bound_target_identity/results.json && test ! -e probes/position_bound_target_identity/RESULTS.md
```

The phase oracle bytes are exact and must not be broadened, substituted, or
rewritten:

```bash
git diff --check edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD && git diff --name-only edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD | python3 -c 'import sys; expected = ["probes/position_bound_target_identity/PROTOCOL.md"]; actual = [line.rstrip("\n") for line in sys.stdin]; raise SystemExit(actual != expected)' && git diff --check 3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD && git diff --name-status 3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD | python3 -c 'import sys; expected = ["A\tprobes/position_bound_target_identity/PROTOCOL.md"]; actual = [line.rstrip("\n") for line in sys.stdin]; raise SystemExit(actual != expected)' && python3 -c 'from pathlib import Path; import sys; text = Path("probes/position_bound_target_identity/PROTOCOL.md").read_text(encoding="utf-8"); authority = text.split("## Block Authority And Candidate Framing\n", 1)[1].split("\nThe exact bounded candidate family", 1)[0]; framing = text.split("### Collision-Safe Token Framing\n", 1)[1].split("\n## Future Manifest Contract", 1)[0]; lineage = text.split("### Wrong-Target Lineage Rules\n", 1)[1].split("\n## Candidate Decisions", 1)[0]; security = text.split("## Future Runner Security Boundary\n", 1)[1].split("\nSource construction, source inspection", 1)[0]; review = text.split("## Review And Completion Policy\n", 1)[1].split("\nThe focused task verifier bytes", 1)[0]; checks = [all(marker in authority for marker in ["canonical live block descriptor", "descriptor-only exclusion", "raw target bytes remain", "candidate token payload", "authorized construction inputs"]), all(marker in framing for marker in ["position-bound-target-identity-token", "ASCII schema version", "unsigned 64-bit big-endian", "0x00", "0x01"]), all(marker in lineage for marker in ["backward_prefix_insertion_bytes", "D[0:a] + I + D[a:b] + D[d:]", "D[0:a] + D[c:]"]), all(marker in security for marker in ["trusted md binary", "--md-binary-sha256", "SHA-256", "shell=False", "env={}", "process hygiene", "not a hostile-code sandbox", "operator OS authority", "outer VM or container"]), all(marker in review for marker in ["edea9e24de21ac178c003b8ce282f1aed32008e3..HEAD", "3771409a39c5554a011349c3abf25ee6c73f2cf1..HEAD", "full authored range", "no unresolved review thread"]), "command text cannot satisfy" in text.split("## Review And Completion Policy\n", 1)[0]]; sys.exit(not all(checks))' && test ! -e probes/position_bound_target_identity/probe.py && test ! -e probes/position_bound_target_identity/cases.json && test ! -e probes/position_bound_target_identity/results.json && test ! -e probes/position_bound_target_identity/RESULTS.md
```

## Honest Conclusion Boundary

This protocol is intentionally narrow. It tests whether a source-only,
block-only bounded context family can reject wrong-target substitutions without
whole-document false-conflict cost.

It does not claim success in advance. The experiment is allowed to fail, and if
all four candidates demote, the only authorized conclusion is evidence-only:
`no_bounded_context_candidate_graduates`.

# F8-4 — `extract_last_json` fence regex strips backticks inside JSON string values

**Status:** OPEN. Filed T8 iter 8 (2026-04-26).

**Surface:** `bench/harness.py` `extract_last_json` line 1560
(`re.sub(r"```(?:json)?\s*\n?", "", text)`) and the text-output
branch of `select_json_envelope_actual` (lines ~1525–1532) which
propagates the corrupted candidate unchanged into
`score_structural_json`.

**Severity:** P1 false-NEGATIVE on json_envelope tasks where the
agent's wrapping envelope embeds a string value containing one or
more backtick triplets. The agent's correct answer is silently
mutated by the harness preprocessor before scoring, so a byte-exact
correct answer is recorded as FAIL.

## Hypothesis

`extract_last_json` preprocesses agent output with a global regex
substitution that strips ` ``` ` markers (with an optional `json`
language tag) anywhere in the text. The intent is to peel off the
canonical LLM output style where the JSON answer is wrapped in a
` ```json … ``` ` fence. The implementation is string-blind: it does
not distinguish between top-level markdown fence boundaries and
backtick triplets that appear inside JSON string values.

When the agent emits an envelope whose `entries[].heading.text` or
`results[].body` field contains a literal ` ``` ` substring (e.g. a
heading naming the language of a code-fence example, or a body
containing fenced code samples), the regex silently strips those
backticks from the string value. The resulting candidate is still
parseable JSON (syntax is preserved — backticks are not quote
characters), but the string content has lost the ` ``` ` markers.
`score_structural_json` then compares actual to expected on
`heading_tree` / `block_text` / etc. and finds a mismatch on every
field whose expected value contained backticks.

## Symmetry to F8-1 / F8-2 / F8-3

The json_envelope failure surface decomposes into layers:

- **F8-1** (CLOSED iter 3): tool-output branch shape match
  (intersection → subset).
- **F8-2** (CLOSED iter 5): preference rule for `extract_last_json`
  candidates (highest-end-position).
- **F8-3** (CLOSED iter 7): depth scanner inside `extract_last_json`
  honors JSON string boundaries on `{`/`}`/`[`/`]`.
- **F8-4** (this finding): regex preprocessor that runs *before*
  the depth scanner is also string-blind, but on a different
  character class — backtick triplets — and it does not break
  parseability, so the failure is silent corruption of string
  content rather than non-enumeration.

F8-4 is fresh: F8-3's string-aware depth scanner does not see
backticks (they aren't structural JSON characters), and the
fence-strip regex runs *before* the depth scanner, so its corruption
is already baked into the text the depth scanner walks. None of the
prior closures fire on this shape.

## Reproduction

```bash
python3 bench/probes/F8-4-fence-regex-strips-string-content/probe.py
```

Pre-fix exit = 1 (live failing trace). Both stages fail:

- **Stage A** — direct `extract_last_json` on agent prose containing
  a heading.text with ` ``` ` returns parseable JSON, but the
  heading.text value has had three backticks stripped:
  `"Example: ```python block"` → `"Example: python block"`.
- **Stage B** — end-to-end harness path: `select_json_envelope_actual`
  → corrupted candidate → `score_structural_json` reports
  `heading_tree [md]: MISMATCH` and FAILs an agent answer that was
  byte-exact correct in the input text.

Post-fix exit = 0 (inert).

## Realism note

Agents extracting structural views of technical Markdown frequently
encounter backticks in heading text, body text, and code-block
content:

- Heading text: `## Example: \`\`\`python` block (tutorial / spec
  doc naming the language of an example).
- Body text: README sections that quote a fenced code block as part
  of an example.
- Block text: `md text-block --json` returning the body of a code
  fence as a string field.

The current 8 json_envelope bench tasks (T1, T5, T9, T11, T16, T19,
T21, T22) do not include this shape in their expected output, so the
trace does not surface in today's benchmark; but the realism is
high enough that any auto-research candidate involving code-fence
extraction tasks will hit it. The probe is filed pre-emptively under
the same precedent as F8-2 and F8-3.

## Closure plan

Either:

1. **Make the fence-strip string-aware**: the regex operates only
   on backticks that fall outside JSON string boundaries (parallel
   to F8-3's string-aware depth scanner).
2. **Anchor the fence-strip to line boundaries**: only strip ` ``` `
   markers that appear at the start of a line (markdown fence
   boundaries are by spec at line starts; backticks inside JSON
   string values typically are not, because JSON encodes embedded
   newlines as `\n` escapes rather than physical newlines).
3. **Drop the fence-strip entirely**: the candidate-enumeration
   path that runs after the failed `json.loads` already finds JSON
   regions inside surrounding prose (including ` ``` ` markers),
   so the fence-strip is not load-bearing for the fenced-JSON case
   — verified by mental trace through `extract_last_json` for
   ` ```json\n{"a":1}\n``` `.

Option (2) or (3) is preferred for simplicity. The closure attribution
probe is `python3 bench/probes/F8-4-fence-regex-strips-string-content/probe.py`;
expect exit 0 on both stages.

## Files

- `probe.py` — standalone reproduction (exit 1 = live, exit 0 = inert)
- `verdict.txt` — recorded probe verdicts (filed and, post-closure, closure)

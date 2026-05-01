# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate benchmark candidates for realistic Markdown document-maintenance tasks. Never mention editing tools, command names, shell commands, benchmark IDs, or benchmark results. Return only a single strict JSON object. Do not use markdown fences around JSON.
```

User:

```text
Create exactly one compact benchmark candidate for an AI coding agent maintaining a real Markdown runbook, design doc, or release checklist.

Hard requirements:
- The edit moves one complete level-3 subsection, with all paragraphs, lists, tables, and fenced blocks it owns, from one level-2 parent section to another level-2 parent section.
- The destination placement is before or after a named sibling subsection.
- The moved subsection has a specific non-generic title.
- The input includes a similarly named level-3 decoy subsection that must stay put.
- The input includes the exact moved heading text inside a fenced code block or blockquote archive that must be ignored.
- The expected output is the full edited Markdown document and is exactly consistent with the input.
- The document is under 70 lines.

Forbidden topic words in every field: setup, configuration, database, certificate, backup, error logging, API rate limit, installation.
Forbidden task shapes: append-only, extraction-only, word counting, pure formatting.

Use an operations or product-delivery topic not covered by the forbidden words. Return JSON with exactly these keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy.
```


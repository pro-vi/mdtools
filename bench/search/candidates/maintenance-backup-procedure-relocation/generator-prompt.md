# Generator Prompt

Model: `magnum-v4-123b-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate for an AI coding agent. It must be a realistic release-engineering or operations Markdown document-maintenance task. The task must require editing a Markdown document by moving one complete second-level subsection (a ## heading and all content it owns) from one top-level # section to another top-level # section, placing it immediately before a named sibling ## subsection. The moved subsection must include one small Markdown table and one fenced shell block. Include one similarly named decoy heading elsewhere, and include one mention of the moved heading inside a fenced code block or quoted archive that must be ignored. The expected output must be the full edited Markdown document. Keep the document under 70 lines. Avoid Installation, Database, Configuration, Setup, append-only edits, extraction-only tasks, word-count tasks, and tasks whose main difficulty is shell quoting. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy. The expected_markdown must be exactly correct and consistent with the input.
```

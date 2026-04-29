# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate. It must be realistic and structurally nontrivial: require editing a Markdown runbook or design document within one specifically scoped section, preserving a similarly named section elsewhere, and ignoring examples inside fenced code or quoted archival notes. Do not create an append-only task. The edit must transform existing content, such as checking exactly one checklist item, removing exactly one obsolete bullet, or moving one note between subsections. Avoid tasks that are mainly about shell quoting. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy. The expected_markdown must be the exact full Markdown file after the edit. Use a path-safe lowercase family_slug with hyphens only.
```

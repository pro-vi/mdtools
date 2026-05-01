# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate. It must be realistic and structurally nontrivial: require reading a Markdown file, ignoring non-document examples such as fenced code or quoted archival notes, and producing a small JSON summary that a release engineer would plausibly need. Avoid trivial word-count or append-one-item tasks. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_json, scorer_policy. The expected_json must be the exact JSON object or array that should be output.
```

# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate. It must be realistic and structurally nontrivial: require targeting a specific section by heading, preserving another similarly named section, and updating nested checklist or table content. Avoid trivial append-one-item tasks. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy. The task should edit a Markdown file in place; expected_markdown is the exact full file after the edit.
```

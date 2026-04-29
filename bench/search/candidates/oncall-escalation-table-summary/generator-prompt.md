# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate for an AI coding agent. It must be a realistic on-call or release-engineering Markdown document-maintenance task. The input must include one real Markdown table under a heading plus at least one fake table inside a fenced code block or blockquote that must be ignored. Include at least one table cell containing an escaped pipe or inline code with a pipe character. The task must require producing a small JSON object summarizing selected rows from the real table. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_json, scorer_policy. The expected_json must be exactly correct and consistent with the input.
```

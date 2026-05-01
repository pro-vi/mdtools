# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate for an AI coding agent. It must be a realistic release-engineering or operations Markdown document-maintenance task. The task must require updating a Markdown document by moving one complete subsection, including its list items and any fenced code block, from one parent section to another parent section. Include at least one similarly named decoy subsection elsewhere and at least one mention of the subsection title inside a fenced code block that must be ignored. The expected output must be the full edited Markdown document. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy. The expected_markdown must be exactly correct and consistent with the input.
```

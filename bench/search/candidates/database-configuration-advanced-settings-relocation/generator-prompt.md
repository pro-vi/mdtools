# Generator Prompt

Model: `magnum-v4-123b-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate for an AI coding agent. It must be a realistic operations, developer-experience, or release-engineering Markdown document-maintenance task. The task must require editing a Markdown document by moving one complete nested subsection or heading-scoped block, including all paragraphs, checklist items, tables, or fenced code blocks it owns, from one parent section to another specific parent section. Include at least one similarly named decoy heading elsewhere, and include at least one mention of the moved heading inside a fenced code block or quoted archive that must be ignored. The expected output must be the full edited Markdown document. Keep the document under 80 lines. Avoid append-only edits, extraction-only tasks, word-count tasks, and tasks whose main difficulty is shell quoting. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy. The expected_markdown must be exactly correct and consistent with the input.
```

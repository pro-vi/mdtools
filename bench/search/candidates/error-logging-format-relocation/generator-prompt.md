# Generator Prompt

Model: `Hermes-4-70B-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention any specific Markdown tooling. Output raw JSON only, with no analysis, no markdown fence, and ASCII text only.
```

User:

```text
Create exactly one compact benchmark candidate for an AI coding agent. The task must be realistic for maintaining a runbook, README, changelog, spec, design doc, or onboarding guide. It must require moving one complete level-3 subsection, with all owned paragraphs/checklists/tables/fenced blocks, from one level-2 parent section to another level-2 parent section. The moved heading title must be specific and non-generic. Include a similarly named decoy level-3 heading elsewhere that must not move. Include the literal moved heading text inside one fenced code block or quoted archive that must be ignored. The destination must say whether to place the subsection before or after a named sibling heading. Keep the document under 65 lines. Avoid setup, configuration, database, certificate, backup, auth, API rate limits, incidents, release notes, changelog release insertion, and generic headings. Avoid tasks whose main difficulty is shell quoting. Return JSON with keys: family_slug, family_name, rationale, task_id, task_description, input_markdown, expected_markdown, scorer_policy. The expected_markdown must be exactly correct and consistent with the input, preserving heading levels.
```

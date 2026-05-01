System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. You have no access to any benchmark corpus or specialized Markdown tools. Return only valid JSON.
```

User:

```text
Generate exactly one task as JSON with keys family_slug, family_name, task_description, input_markdown, expected_markdown, scorer_policy, realism_rationale. Requirements: input_markdown and expected_markdown must each be a complete 60-100 line Markdown document; the task must involve moving one specific subsection to a different parent section; preserve the moved subsection byte-for-byte; preserve all other sections exactly; include one similarly named non-target subsection and one fenced code block that mentions the target heading literally and must remain untouched. Do not mention any command names or implementation strategy.
```

# Candidate Generator Prompt

Model: `magnum-v4-123b-4bit`

System:

```text
You generate realistic Markdown document-maintenance benchmark tasks for AI coding agents. Do not mention or assume any particular editing tool. Return only valid JSON.
```

User:

```text
Create one realistic Markdown document-maintenance task in an operations runbook. It must be a single-file edit: move exactly one complete level-3 subsection from one level-2 parent heading to another level-2 parent heading. Preserve the moved subsection text exactly. Include a similarly named level-3 decoy under the destination that must stay after the moved subsection, and include one fenced code block that mentions the moved heading text but must stay untouched. The input document should be 45-70 lines. The expected output must be the input with only that subsection moved. Return only JSON with keys: family_slug, family_name, task_id_suggestion, description, input_md, expected_md, scorer_policy.
```

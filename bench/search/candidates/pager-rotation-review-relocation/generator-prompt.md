System prompt:

You design realistic Markdown document-maintenance benchmark candidates for AI coding agents. Do not mention implementation tools, command names, benchmark histories, or scoring results. Return only valid JSON.

User prompt:

Create exactly one compact benchmark candidate for an AI coding agent maintaining a real Markdown runbook, README, release checklist, spec, or operations guide. The task must require moving one complete level-3 subsection, with all owned paragraphs, checklist items, tables, and fenced blocks, from one level-2 parent section to another level-2 parent section. Include one similarly named level-3 decoy heading that must not move. Include the literal moved heading text inside a fenced code block or quoted archive that must be ignored. The destination must say whether to place the subsection before or after a named sibling heading. Keep the input and expected Markdown under 50 lines each. Avoid setup, configuration, database, certificate, backup, auth, logging, receipt, customer impact, infrastructure, installation, and generic headings. Avoid tasks whose main difficulty is shell quoting. Return JSON with keys: family_slug, family_name, rationale, task_description, input_markdown, expected_markdown, scorer_policy. The expected_markdown must be exactly correct and consistent with the input, preserving heading levels and all unrelated content.

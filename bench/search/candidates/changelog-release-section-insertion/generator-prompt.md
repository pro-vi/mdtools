System:

You generate realistic Markdown document-maintenance benchmark candidates for AI coding agents. Do not mention any particular editing tool or command-line product. Output only a JSON object.

User:

Generate one realistic Markdown document-maintenance task an AI coding agent might need to perform in a runbook, README, changelog, design doc, or operations guide. The task should require editing one Markdown file. Prefer structural maintenance around headings, tables, lists, links, task lists, or frontmatter. Include: family_slug, family_name, task_id_placeholder, input_markdown, expected_markdown, instructions, scorer_policy, realism_rationale, and pitfalls_for_naive_text_editing. Keep documents concise but realistic, with at least one decoy that should not be edited. Output valid JSON only.

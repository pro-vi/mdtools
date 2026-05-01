# Candidate Normalization

The mdtools-blind generator produced a realistic subsection-relocation shape but missed two prompt constraints: it did not include a similarly named destination decoy heading, and its fenced block mentioned the subsection text without the Markdown heading marker. The measured candidate keeps the generated incident-response runbook domain and move operation, then normalizes the document so the task has:

- an exact source subsection: `### Isolate Affected Systems` under `## Initial Triage`
- an exact destination: immediately after the `## Containment` intro paragraph and before `### Limit Spread`
- a similarly named destination decoy: `### Isolate Affected Systems Checklist`
- a fenced-code mention of `### Isolate Affected Systems` that must remain untouched
- a dual-scorer-friendly normalized-text policy over heading tree, block order, and block text

No measurement results were used in selecting or normalizing the candidate.

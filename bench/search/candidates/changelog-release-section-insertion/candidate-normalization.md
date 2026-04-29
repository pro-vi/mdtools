# Candidate Normalization

The generator produced a realistic changelog insertion task but left the new release bullets implicit in the instruction while placing concrete bullets in the expected output. The normalized task names the exact bullets so the benchmark is not asking the agent to guess hidden content.

The normalized input also adds two common real-world decoys:

- a fenced Markdown example that contains a fake `## [0.4.0]` heading
- a release template section that should remain unchanged

The candidate remains in the generated family: add a new release section at the top of an existing changelog while preserving surrounding release entries.

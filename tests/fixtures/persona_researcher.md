# Effects of Markdown Tooling on Agent Performance

## Abstract

This paper examines the impact of structural markdown tools on LLM agent task completion rates[^1]. We find significant improvements when agents use typed interfaces rather than raw text manipulation.

## Introduction

As noted by Smith [@smith2020], the fundamental challenge in agent tooling is bridging the gap between token-level operations and document-level semantics. This finding was corroborated by later work [see @doe2021, pp. 15-23].

> The most significant barrier to agent adoption is not capability but interface design.
>
> — Johnson et al. [@johnson2022]

Previous approaches[^2] relied on line-number addressing, which is fragile under concurrent edits.

## Methods

### Participants

We recruited 50 LLM agents across three model families[^3].

### Procedure

Each agent completed four benchmark tasks:

1. Extract document outline
2. Insert content at a structural location
3. Replace a named section
4. Perform targeted text substitution

### Analysis

Results were scored using a structural diff policy with normalized text comparison.

## Results

The mdtools condition outperformed both Unix tools (p < 0.001) and function-call APIs (p < 0.01)[^4].

| Condition | Accuracy | Tokens | Time |
|-----------|----------|--------|------|
| Unix tools | 62% | 4,521 | 34s |
| mdtools | 94% | 1,203 | 12s |
| Function calls | 88% | 2,847 | 18s |

## Discussion

These findings suggest that structural awareness — not just semantic understanding — is critical for agent performance. As @jones2023 argues, "the right abstraction level determines task success."

## Conclusion

We conclude that markdown-aware CLI tools significantly improve agent task performance.

## References

[^1]: Performance measured as task completion accuracy across the T1-T4 benchmark suite.
[^2]: See Appendix A for a survey of prior approaches.
[^3]: Model families: GPT-4, Claude 3, and Gemini Pro.
[^4]: All p-values computed using paired t-tests with Bonferroni correction.

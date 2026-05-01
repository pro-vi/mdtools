# Candidate Normalization

The mdtools-blind generator produced a realistic subsection-relocation task, but the raw document did not satisfy the prompt constraints requiring a similarly named decoy heading and an ignored fenced/quoted mention of the moved heading.

Normalization kept the generator's core task shape and changed only the document fixture:

- renamed the family to `getting-started-installation-relocation`;
- made the document a developer-onboarding guide, which matches the generated setup/install theme;
- added a fenced historical outline before the real `### Installation` heading containing a literal `### Installation` mention that must not be treated as a heading;
- added a decoy `### Installation Notes` subsection under Troubleshooting;
- enriched the moved subsection with a checklist and fenced command block so the move must preserve a complete heading-scoped block;
- specified insertion immediately before `### Runtime Versions` under `## Prerequisites`.

No gap measurements had been run before this normalization.

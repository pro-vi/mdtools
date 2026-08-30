# Parser-dropped footnote definitions

Unreferenced footnote definitions are omitted by the pinned Comrak parser before
`DocumentIndex` is built. They therefore do not appear in `map` output or search
evidence. Referenced footnote definitions retain source-position ownership and
remain searchable.

A complete fix needs one parser-boundary decision: either preserve unreferenced
definitions as typed index nodes, or define a source-gap evidence kind that does
not pretend the parser retained Markdown structure. Search must not guess a
mutable target address for parser-dropped source.

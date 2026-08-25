use mdtools::block;
use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::fingerprint::TargetEtagGuard;
use mdtools::locate::{locate, locate_line};
use mdtools::model::{BlockKind, HeadingMatchMode, SectionKind};
use mdtools::section::{SectionIndex, SectionTarget};
use mdtools::table;
use mdtools::task::{self, TaskLoc};

const DOC: &str = "# Title\n\nIntro paragraph.\n\n## Plan\n\n- [ ] parent\n  - [ ] child\n\n```rust\nlet answer = 42;\n```\n\nTail paragraph.\n";

/// Byte offset of the first occurrence of `needle`, so tests name source text
/// rather than hand-counted numbers.
fn offset_of(source: &str, needle: &str) -> u32 {
    source.find(needle).expect("needle present in source") as u32
}

#[test]
fn heading_offset_resolves_to_the_heading_block_with_its_blocks_etag() {
    let document = Document::parse(DOC).unwrap();
    let located = locate(&document, offset_of(DOC, "Plan")).unwrap();

    let hit = located.block.expect("heading block");
    assert_eq!(hit.kind, BlockKind::Heading);
    let listed = &block::blocks(&document)[hit.index as usize];
    assert_eq!(&hit, listed);
    assert_eq!(hit.etag, listed.etag);
    assert!(located.task.is_none());
}

#[test]
fn paragraph_offset_resolves_to_the_paragraph_with_no_task() {
    let document = Document::parse(DOC).unwrap();
    let located = locate(&document, offset_of(DOC, "Intro paragraph.")).unwrap();

    let hit = located.block.expect("paragraph block");
    assert_eq!(hit.kind, BlockKind::Paragraph);
    assert!(hit.preview.contains("Intro paragraph."));
    assert!(located.task.is_none());
}

#[test]
fn code_fence_body_offset_resolves_to_the_code_block() {
    let document = Document::parse(DOC).unwrap();
    let located = locate(&document, offset_of(DOC, "let answer = 42;")).unwrap();

    let hit = located.block.expect("code block");
    assert_eq!(hit.kind, BlockKind::CodeFence);
    assert!(document
        .slice(&hit.span)
        .unwrap()
        .contains("let answer = 42;"));
}

#[test]
fn nested_task_offset_resolves_to_the_innermost_item() {
    let document = Document::parse(DOC).unwrap();
    let located = locate(&document, offset_of(DOC, "child")).unwrap();

    let hit = located.task.expect("task item");
    assert_eq!(hit.summary_text, "child");
    assert_eq!(hit.loc.child_path().len(), 2, "loc: {}", hit.loc);
    let direct = task::task(&document, &hit.loc).unwrap();
    assert_eq!(direct.task.etag, hit.etag);
    assert_eq!(direct.task.span, hit.span);

    let parent = locate(&document, offset_of(DOC, "parent")).unwrap();
    let parent = parent.task.expect("task item");
    assert_eq!(parent.summary_text, "parent");
    assert_ne!(parent.loc, hit.loc);
    assert!(parent.depth < hit.depth);
}

#[test]
fn blank_line_between_blocks_is_ok_with_no_targets() {
    let document = Document::parse(DOC).unwrap();
    // The newline that ends the blank line separating the two paragraphs.
    let blank = offset_of(DOC, "Intro paragraph.\n\n") + "Intro paragraph.\n".len() as u32;
    assert_eq!(DOC.as_bytes()[blank as usize], b'\n');

    let located = locate(&document, blank).unwrap();
    assert!(located.block.is_none());
    assert!(located.task.is_none());
}

#[test]
fn crlf_offsets_resolve_to_the_same_blocks_as_the_lf_twin() {
    let crlf = DOC.replace('\n', "\r\n");
    let lf_document = Document::parse(DOC).unwrap();
    let crlf_document = Document::parse(crlf.as_str()).unwrap();

    for needle in ["Title", "Intro paragraph.", "Plan", "child", "let answer"] {
        let lf = locate(&lf_document, offset_of(DOC, needle))
            .unwrap()
            .block
            .expect(needle);
        let with_crlf = locate(&crlf_document, offset_of(&crlf, needle))
            .unwrap()
            .block
            .expect(needle);
        assert_eq!(lf.index, with_crlf.index, "block index for {needle:?}");
        assert_eq!(lf.kind, with_crlf.kind, "block kind for {needle:?}");
    }
}

#[test]
fn locate_line_maps_lines_to_the_same_targets() {
    let document = Document::parse(include_str!("fixtures/frontmatter.md")).unwrap();
    assert!(locate_line(&document, 1).unwrap().block.is_none());

    let source = include_str!("fixtures/frontmatter.md");
    let heading_offset = offset_of(source, "\n# ") + 1;
    let heading_line = document.byte_to_line(heading_offset);
    let by_line = locate_line(&document, heading_line).unwrap();
    let by_byte = locate(&document, heading_offset).unwrap();

    let by_line = by_line.block.expect("heading block");
    assert_eq!(by_line.kind, BlockKind::Heading);
    assert_eq!(by_line, by_byte.block.expect("heading block"));
}

#[test]
fn positions_outside_the_document_are_errors() {
    let document = Document::parse(DOC).unwrap();

    assert!(matches!(
        locate(&document, DOC.len() as u32),
        Err(CoreError::ByteOffsetOutOfRange { byte_offset, source_len })
            if byte_offset as usize == DOC.len() && source_len as usize == DOC.len()
    ));
    assert!(matches!(
        locate_line(&document, 0),
        Err(CoreError::LineOutOfRange { line: 0, .. })
    ));
    let past_end = document.line_count() + 1;
    assert!(matches!(
        locate_line(&document, past_end),
        Err(CoreError::LineOutOfRange { line, .. }) if line == past_end
    ));
}

#[test]
fn located_task_loc_round_trips_through_the_task_read_path() {
    let document = Document::parse(include_str!("fixtures/nested_tasks.md")).unwrap();
    let source = include_str!("fixtures/nested_tasks.md");
    let all = task::tasks(&document, &Default::default()).unwrap();

    for record in &all {
        let inside = record.span.byte_start;
        let located = locate(&document, inside).unwrap().task.expect("task item");
        assert_eq!(
            located.loc, record.loc,
            "locate resolved byte {inside} to {} but that byte starts {}",
            located.loc, record.loc
        );
        assert_eq!(located.span, record.span);
        assert_eq!(located.etag, record.etag);
        assert_eq!(
            located.loc.to_string().parse::<TaskLoc>().unwrap(),
            located.loc
        );
        assert!(source.len() > inside as usize);
    }
}

const NESTED: &str = "---\ntitle: Ledger\n---\n\nLead paragraph.\n\n## Top\n\nUnder top.\n\n### Sub\n\nUnder sub.\n\nStill under sub.\n";

#[test]
fn section_is_the_deepest_heading_owning_the_block() {
    let document = Document::parse(NESTED).unwrap();
    let located = locate(&document, offset_of(NESTED, "Under sub.")).unwrap();

    let section = located.section.expect("section");
    assert_eq!(section.heading.as_ref().unwrap().text, "Sub");
    assert_eq!(section.depth, 3);

    let resolved = SectionIndex::new(&document)
        .resolve(&SectionTarget::heading("Sub", None, HeadingMatchMode::Exact).unwrap())
        .unwrap();
    assert_eq!(section.etag, resolved.etag);
    assert_eq!(section.span, resolved.span);
}

#[test]
fn paragraph_before_the_first_heading_is_the_preamble() {
    let document = Document::parse(NESTED).unwrap();
    let located = locate(&document, offset_of(NESTED, "Lead paragraph.")).unwrap();

    let section = located.section.expect("section");
    assert_eq!(section.kind, SectionKind::Preamble);
    let resolved = SectionIndex::new(&document)
        .resolve(&SectionTarget::preamble())
        .unwrap();
    assert_eq!(section.etag, resolved.etag);
}

#[test]
fn blank_line_inside_a_section_keeps_the_section_without_a_block() {
    let document = Document::parse(NESTED).unwrap();
    let blank = offset_of(NESTED, "Under sub.\n\n") + "Under sub.\n".len() as u32;
    assert_eq!(NESTED.as_bytes()[blank as usize], b'\n');

    let located = locate(&document, blank).unwrap();
    assert!(located.block.is_none());
    let section = located.section.expect("section");
    assert_eq!(section.heading.as_ref().unwrap().text, "Sub");
}

#[test]
fn frontmatter_offsets_have_no_section() {
    let document = Document::parse(NESTED).unwrap();
    let located = locate(&document, offset_of(NESTED, "title: Ledger")).unwrap();

    assert!(located.block.is_none());
    assert!(located.section.is_none());
    assert!(located.task.is_none());
}

const TABLE_DOC: &str = include_str!("fixtures/table.md");

#[test]
fn table_data_row_offset_resolves_to_a_row_the_mutations_accept() {
    let document = Document::parse(TABLE_DOC).unwrap();
    let located = locate(&document, offset_of(TABLE_DOC, "Beta")).unwrap();

    let block = located.block.expect("table block");
    assert_eq!(block.kind, BlockKind::Table);
    let row = located.table_row.expect("table row");
    assert_eq!(row.row_index, 1);
    assert_eq!(row.table_block_index, block.index);
    assert_eq!(row.table_etag, block.etag);
    assert!(document.slice(&row.span).unwrap().contains("Beta"));

    let edit = table::prepare_replace_row(
        &document,
        row.table_block_index,
        row.row_index,
        Some(&TargetEtagGuard::from(row.table_etag.clone())),
    )
    .unwrap()
    .replace("| Beta | 250 |\n")
    .unwrap();
    assert!(edit.content.contains("| Beta | 250 |"));
    assert!(!edit.content.contains("| Beta | 200 |"));

    // The same guard is stale once the row has been edited.
    let edited = Document::parse(edit.content).unwrap();
    assert!(matches!(
        table::prepare_replace_row(
            &edited,
            row.table_block_index,
            row.row_index,
            Some(&TargetEtagGuard::from(row.table_etag.clone())),
        ),
        Err(CoreError::TargetEtagMismatch { .. })
    ));
}

#[test]
fn table_header_and_separator_hit_the_table_but_no_row() {
    let document = Document::parse(TABLE_DOC).unwrap();

    for needle in ["| Name | Value |", "|------|-------|"] {
        let located = locate(&document, offset_of(TABLE_DOC, needle)).unwrap();
        assert_eq!(
            located.block.expect(needle).kind,
            BlockKind::Table,
            "block for {needle:?}"
        );
        assert!(located.table_row.is_none(), "row for {needle:?}");
    }
}

#[test]
fn non_table_positions_carry_no_table_row() {
    let document = Document::parse(TABLE_DOC).unwrap();
    let located = locate(&document, offset_of(TABLE_DOC, "Summary paragraph.")).unwrap();
    assert!(located.table_row.is_none());
}

// --- Invariants across representations ---

#[test]
fn the_two_section_lookups_agree_on_every_block() {
    // section_for_block reads block_indices; section_for_byte reads the span.
    // SectionIndex derives those two from the same heading-level cut, so a
    // click on a block and a click on the blank line beside it must never
    // report different sections.
    let document = Document::parse(include_str!("fixtures/footnote_midbody.md")).unwrap();
    let index = SectionIndex::new(&document);
    assert!(
        document
            .blocks()
            .windows(2)
            .any(|pair| pair[1].span.byte_start < pair[0].span.byte_start),
        "fixture must emit a block out of source order, or it cannot catch the bug"
    );

    for block in document.blocks() {
        let by_block = index
            .section_for_block(block.index)
            .expect("owning section");
        let by_byte = index
            .section_for_byte(block.span.byte_start)
            .expect("containing section");
        assert_eq!(
            by_block.etag, by_byte.etag,
            "block {} disagrees: {:?} vs {:?}",
            block.index, by_block.heading, by_byte.heading
        );
        assert_eq!(by_block.span, by_byte.span);
    }
}

#[test]
fn crlf_tasks_and_sections_resolve_like_their_lf_twins() {
    let lf = include_str!("fixtures/nested_tasks.md").replace("\r\n", "\n");
    let crlf = lf.replace('\n', "\r\n");
    let lf_document = Document::parse(lf.as_str()).unwrap();
    let crlf_document = Document::parse(crlf.as_str()).unwrap();

    for needle in ["Grandchild task", "Sub-task B", "Ordered child"] {
        let from_lf = locate(&lf_document, offset_of(&lf, needle)).unwrap();
        let from_crlf = locate(&crlf_document, offset_of(&crlf, needle)).unwrap();

        let lf_task = from_lf.task.expect(needle);
        let crlf_task = from_crlf.task.expect(needle);
        assert_eq!(lf_task.loc, crlf_task.loc, "task loc for {needle:?}");
        assert_eq!(lf_task.depth, crlf_task.depth, "task depth for {needle:?}");
        assert_eq!(
            lf_task.summary_text, crlf_task.summary_text,
            "summary for {needle:?}"
        );

        let lf_heading = from_lf.section.expect(needle).heading.unwrap().text;
        let crlf_heading = from_crlf.section.expect(needle).heading.unwrap().text;
        assert_eq!(lf_heading, crlf_heading, "section for {needle:?}");
    }
}

#[test]
fn an_offset_inside_a_multibyte_character_still_resolves() {
    let source = include_str!("fixtures/utf8.md");
    let document = Document::parse(source).unwrap();
    let start = offset_of(source, "日本語");
    let interior = start + 1;
    assert!(!source.is_char_boundary(interior as usize));

    let at_boundary = locate(&document, start).unwrap();
    let inside = locate(&document, interior).unwrap();
    assert_eq!(
        at_boundary.block.expect("block").index,
        inside.block.expect("block").index
    );
    assert_eq!(
        at_boundary.section.expect("section").etag,
        inside.section.expect("section").etag
    );
}

#[test]
fn walking_every_line_of_a_document_never_errors() {
    // LineIndex counts the position after a final newline as a line. It holds
    // nothing, so it resolves to an empty result rather than an out-of-range
    // byte: `for line in 1..=line_count()` is the loop locate_line exists to
    // serve, and it must not need a special case at the end.
    let document = Document::parse(DOC).unwrap();
    assert!(DOC.ends_with('\n'));
    let last = document.line_count();

    for line in 1..=last {
        assert!(
            locate_line(&document, line).is_ok(),
            "line {line} of {last} errored"
        );
    }
    assert_eq!(locate_line(&document, last).unwrap(), Default::default());
    assert!(locate_line(&document, last - 1).unwrap().block.is_some());
}

// --- Regressions from review of PR #41 ---

const FOOTNOTE_DOC: &str = include_str!("fixtures/footnote_midbody.md");

#[test]
fn every_block_is_reachable_when_the_parser_emits_them_out_of_source_order() {
    // comrak hoists footnote definitions to the end of the root's children, so
    // document.blocks() is two ascending runs. A binary search over that
    // reported "no block here" for ordinary content.
    let document = Document::parse(FOOTNOTE_DOC).unwrap();

    for block in document.blocks() {
        let located = locate(&document, block.span.byte_start)
            .unwrap()
            .block
            .unwrap_or_else(|| {
                panic!(
                    "block {} ({:?}) at byte {} resolved to nothing",
                    block.index, block.kind, block.span.byte_start
                )
            });
        assert_eq!(located.index, block.index);
    }

    // And every byte inside a block, not only its first.
    for offset in 0..FOOTNOTE_DOC.len() as u32 {
        let expected = document
            .blocks()
            .iter()
            .find(|block| block.span.byte_start <= offset && offset < block.span.byte_end)
            .map(|block| block.index);
        let got = locate(&document, offset)
            .unwrap()
            .block
            .map(|hit| hit.index);
        assert_eq!(got, expected, "byte {offset}");
    }
}

#[test]
fn a_footnote_definition_belongs_to_the_section_its_bytes_sit_in() {
    let document = Document::parse(FOOTNOTE_DOC).unwrap();
    let definition = document
        .blocks()
        .iter()
        .find(|block| block.kind == BlockKind::FootnoteDefinition)
        .expect("fixture has a footnote definition");

    let located = locate(&document, definition.span.byte_start).unwrap();
    let section = located.section.expect("section");
    assert_eq!(
        section.heading.as_ref().unwrap().text,
        "Notes",
        "the definition's bytes sit under `# Notes`, whatever its block index is"
    );
}

#[test]
fn a_footnote_first_document_resolves_instead_of_panicking() {
    // build_preamble took its bounds from the first and last block in vec
    // order, which here gives byte_start > byte_end and panicked the slice.
    let document = Document::parse("[^1]: first note\n\nbody[^1]\n").unwrap();
    let located = locate(&document, 0).unwrap();
    assert!(located.block.is_some());
    assert!(located.section.is_some());
}

#[test]
fn a_located_section_can_re_address_itself_when_headings_repeat() {
    let source = "## Notes\n\nfirst\n\n## Notes\n\nsecond\n";
    let document = Document::parse(source).unwrap();
    let located = locate(&document, offset_of(source, "second")).unwrap();
    let section = located.section.expect("section");

    let selector = SectionTarget::heading(
        section.selector.heading_text.clone().unwrap(),
        section.selector.occurrence,
        section.selector.match_mode,
    )
    .unwrap();
    let resolved = SectionIndex::new(&document).resolve(&selector).unwrap();

    assert_eq!(resolved.etag, section.etag);
    assert_eq!(
        resolved.heading.as_ref().unwrap().block_index,
        section.heading.as_ref().unwrap().block_index
    );
}

#[test]
fn an_indented_line_resolves_to_the_block_it_indents() {
    // Up to three spaces of indentation are legal and belong to no block, so a
    // block's span starts after them.
    for source in [
        "   para\n",
        "   - item\n",
        "   > quote\n",
        "   | a | b |\n   |---|---|\n   | x | y |\n",
    ] {
        let document = Document::parse(source).unwrap();
        assert!(
            locate_line(&document, 1).unwrap().block.is_some(),
            "line 1 of {source:?} resolved to nothing"
        );
    }
}

#[test]
fn the_preamble_owns_the_blank_line_before_the_first_heading() {
    let source = "One.\n\nTwo.\n\n## H\n\nx\n";
    let document = Document::parse(source).unwrap();
    let gap = offset_of(source, "Two.\n\n") + "Two.\n".len() as u32;
    assert_eq!(source.as_bytes()[gap as usize], b'\n');

    let located = locate(&document, gap).unwrap();
    assert!(located.block.is_none());
    assert_eq!(
        located.section.expect("section").kind,
        SectionKind::Preamble,
        "section: None must mean frontmatter, and this is not frontmatter"
    );
}

#[test]
fn a_click_at_the_end_of_a_table_row_still_names_that_row() {
    // A row's span excludes its line ending because that is what the mutation
    // splices over; containment has to include it, or an end-of-line click
    // falls back to a whole-table edit.
    for source in [TABLE_DOC.to_string(), TABLE_DOC.replace('\n', "\r\n")] {
        let document = Document::parse(source.as_str()).unwrap();
        let alpha = offset_of(&source, "Alpha");
        let row = locate(&document, alpha).unwrap().table_row.expect("row");
        assert_eq!(row.row_index, 0, "Alpha must not be the table's last row");

        let at_end = locate(&document, row.span.byte_end)
            .unwrap()
            .table_row
            .expect("row at its terminating newline");
        assert_eq!(at_end.row_index, row.row_index);
        assert_eq!(at_end.span, row.span);
    }
}

#[test]
fn the_newline_after_a_blocks_last_line_belongs_to_no_block() {
    // Block spans exclude their trailing newline, so this offset is outside
    // every block. Pinned because the Located.block doc names it as a cause of
    // absence, alongside frontmatter and between-block gaps.
    let document = Document::parse(TABLE_DOC).unwrap();
    let last_row_end = offset_of(TABLE_DOC, "| Beta | 200 |") + "| Beta | 200 |".len() as u32;
    assert_eq!(TABLE_DOC.as_bytes()[last_row_end as usize], b'\n');

    let located = locate(&document, last_row_end).unwrap();
    assert!(located.block.is_none());
    assert!(located.table_row.is_none());
    assert!(located.section.is_some(), "the section still answers");
}

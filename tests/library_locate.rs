use mdtools::block;
use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::locate::{locate, locate_line};
use mdtools::model::BlockKind;
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
            if byte_offset as usize == DOC.len() && source_len == DOC.len()
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
            located.loc.to_string().parse::<TaskLoc>().unwrap(),
            located.loc
        );
        assert!(
            all.iter().any(|candidate| candidate.loc == located.loc),
            "locate produced a loc no task read reports: {}",
            located.loc
        );
        assert!(source.len() > inside as usize);
    }
}

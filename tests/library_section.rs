use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::model::{HeadingMatchMode, SectionKind};
use mdtools::section::{SectionIndex, SectionTarget};
use mdtools::section_edit;

fn heading_selector(text: &str, occurrence: Option<u32>) -> SectionTarget {
    SectionTarget::heading(text, occurrence, HeadingMatchMode::Exact).unwrap()
}

#[test]
fn outline_and_section_share_span_and_etag() {
    let document = Document::parse(include_str!("fixtures/basic.md")).unwrap();
    let index = SectionIndex::new(&document);
    let outline = index.outline();
    let first = &outline[0];
    let section = index
        .resolve(&heading_selector(&first.heading.text, None))
        .unwrap();

    assert_eq!(section.span, first.section_span);
    assert_eq!(section.etag, first.etag);
    assert_eq!(
        document.try_slice(&section.span).unwrap(),
        document.slice(&section.span).unwrap()
    );
}

#[test]
fn duplicate_heading_requires_occurrence() {
    let document = Document::parse(include_str!("fixtures/duplicate_headings.md")).unwrap();
    let index = SectionIndex::new(&document);
    let duplicate = index
        .outline()
        .iter()
        .map(|entry| entry.heading.text.as_str())
        .find(|heading| {
            index
                .outline()
                .iter()
                .filter(|entry| entry.heading.text == *heading)
                .count()
                > 1
        })
        .unwrap()
        .to_string();

    assert!(matches!(
        index.resolve(&heading_selector(&duplicate, None)),
        Err(CoreError::DuplicateHeading { .. })
    ));
    assert!(index
        .resolve(&heading_selector(&duplicate, Some(2)))
        .is_ok());
}

#[test]
fn zero_occurrence_is_rejected_at_the_core_boundary() {
    assert!(matches!(
        SectionTarget::heading("Title", Some(0), HeadingMatchMode::Exact),
        Err(CoreError::InvalidSelector(_))
    ));
}

#[test]
fn resolved_section_cannot_be_reused_with_another_document() {
    let original = Document::parse("# One\n\nbody\n").unwrap();
    let other = Document::parse("# Two\n\nbody\n").unwrap();
    let resolved = SectionIndex::new(&original)
        .resolve(&heading_selector("One", None))
        .unwrap();

    assert!(matches!(
        section_edit::delete(&other, resolved, None),
        Err(CoreError::DocumentRevisionMismatch { .. })
    ));
}

#[test]
fn section_for_block_and_byte_pick_the_innermost_owner() {
    let source = "Lead.\n\n## Top\n\nUnder top.\n\n### Sub\n\nUnder sub.\n\nTail.\n";
    let document = Document::parse(source).unwrap();
    let index = SectionIndex::new(&document);

    let deepest = document
        .blocks()
        .iter()
        .find(|block| document.slice(&block.span).unwrap().contains("Under sub."))
        .unwrap();
    let by_block = index.section_for_block(deepest.index).unwrap();
    assert_eq!(by_block.heading.as_ref().unwrap().text, "Sub");
    assert_eq!(
        by_block.etag,
        index.resolve(&heading_selector("Sub", None)).unwrap().etag
    );

    // The blank line after "Under sub." belongs to no block but stays in "Sub".
    let blank = (source.find("Under sub.").unwrap() + "Under sub.\n".len()) as u32;
    assert_eq!(source.as_bytes()[blank as usize], b'\n');
    assert_eq!(
        index.section_for_byte(blank).unwrap().heading.unwrap().text,
        "Sub"
    );

    let lead = document.blocks()[0].index;
    assert_eq!(
        index.section_for_block(lead).unwrap().kind,
        SectionKind::Preamble
    );

    let inside_top = source.find("Under top.").unwrap() as u32;
    assert_eq!(
        index
            .section_for_byte(inside_top)
            .unwrap()
            .heading
            .unwrap()
            .text,
        "Top"
    );
}

use mdtools::core_error::CoreError;
use mdtools::model::{HeadingMatchMode, SectionSelector, SectionSelectorKind};
use mdtools::parser::ParsedDocument;
use mdtools::section::SectionIndex;

fn heading_selector(text: &str, occurrence: Option<u32>) -> SectionSelector {
    SectionSelector {
        kind: SectionSelectorKind::HeadingText,
        heading_text: Some(text.to_string()),
        occurrence,
        match_mode: HeadingMatchMode::Exact,
    }
}

#[test]
fn outline_and_section_share_span_and_etag() {
    let document = ParsedDocument::parse(include_str!("fixtures/basic.md").to_string()).unwrap();
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
        document.slice(&section.span)
    );
}

#[test]
fn duplicate_heading_requires_occurrence() {
    let document =
        ParsedDocument::parse(include_str!("fixtures/duplicate_headings.md").to_string()).unwrap();
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

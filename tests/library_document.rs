use mdtools::core_error::CoreError;
use mdtools::model::SourceSpan;
use mdtools::parser::ParsedDocument;
use mdtools::revision::verify_source_revision;

#[test]
fn document_revision_tracks_exact_source_bytes() {
    let lf = ParsedDocument::parse("# Title\nbody\n".to_string()).unwrap();
    let crlf = ParsedDocument::parse("# Title\r\nbody\r\n".to_string()).unwrap();

    assert_ne!(lf.revision(), crlf.revision());
    assert_eq!(lf.revision().as_str().len(), 64);
}

#[test]
fn checked_slice_accepts_parser_spans_and_rejects_untrusted_ranges() {
    let doc = ParsedDocument::parse("éclair\n".to_string()).unwrap();
    let whole = SourceSpan {
        line_start: 1,
        line_end: 1,
        byte_start: 0,
        byte_end: 7,
    };
    assert_eq!(doc.try_slice(&whole).unwrap(), "éclair");

    let invalid = SourceSpan {
        line_start: 1,
        line_end: 1,
        byte_start: 1,
        byte_end: 2,
    };
    assert!(matches!(
        doc.try_slice(&invalid),
        Err(CoreError::InvalidSpan { .. })
    ));
}

#[test]
fn stale_document_revision_is_rejected() {
    let document = ParsedDocument::parse("before\n".to_string()).unwrap();
    verify_source_revision("before\n", document.revision()).unwrap();
    assert!(matches!(
        verify_source_revision("after\n", document.revision()),
        Err(CoreError::DocumentRevisionMismatch { .. })
    ));
}

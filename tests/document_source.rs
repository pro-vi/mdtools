use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::revision::DocumentRevision;
use mdtools::{LineEndingStyle, SourceSpan};

#[test]
fn document_source_authority_preserves_bytes_coordinates_and_revision() {
    let cases = [
        ("", vec![Some(0), None], LineEndingStyle::Lf),
        ("one\n", vec![Some(0), Some(4), None], LineEndingStyle::Lf),
        (
            "one\r\ntwo",
            vec![Some(0), Some(5), None],
            LineEndingStyle::Crlf,
        ),
        (
            "one\ntwo\r\n世界",
            vec![Some(0), Some(4), Some(9), None],
            LineEndingStyle::Mixed,
        ),
    ];

    for (source, line_starts, line_endings) in cases {
        let document = Document::parse(source).unwrap();
        assert_eq!(document.source(), source);
        assert_eq!(document.revision(), &DocumentRevision::for_source(source));
        assert_eq!(document.line_count(), line_starts.len() as u32 - 1);
        assert_eq!(document.line_ending_style(), line_endings);
        for (line, expected) in line_starts.into_iter().enumerate() {
            assert_eq!(document.line_to_byte(line as u32 + 1), expected);
        }
        let full = document.span_for_byte_range(0, source.len() as u32);
        assert_eq!(document.slice(&full).unwrap(), source);
    }
}

#[test]
fn document_source_authority_preserves_all_parse_policies() {
    let source = "---\nk: value\n---\n\n# Title\n\nbody";
    let documents = [
        Document::parse(source).unwrap(),
        Document::parse_for_frontmatter(source).unwrap(),
        Document::parse_for_frontmatter_mutation(source).unwrap(),
    ];

    for document in &documents {
        assert_eq!(document.source(), source);
        assert_eq!(document.revision(), &DocumentRevision::for_source(source));
        assert_eq!(document.map().unwrap(), documents[0].map().unwrap());
    }
}

#[test]
fn document_source_rejects_invalid_utf8_slices() {
    let document = Document::parse("é").unwrap();
    assert!(matches!(
        document.slice(&SourceSpan {
            line_start: 1,
            line_end: 1,
            byte_start: 1,
            byte_end: 2,
        }),
        Err(CoreError::InvalidSpan {
            reason: "offset is not a UTF-8 character boundary",
            ..
        })
    ));
}

use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;
use mdtools::search::{self, SearchQuery};

fn exact_source<'a>(document: &'a Document, start: u32, end: u32) -> &'a str {
    &document.source()[start as usize..end as usize]
}

#[test]
fn search_hit_etag_matches_exact_source_bytes() {
    let document = Document::parse("alpha needle omega").unwrap();
    let matches = search::search(&document, &SearchQuery::literal("needle"));

    assert_eq!(matches.len(), 1);
    let found = &matches[0];
    let source = exact_source(
        &document,
        found.match_span.byte_start,
        found.match_span.byte_end,
    );
    assert_eq!(source, "needle");
    assert_eq!(found.etag, TargetEtag::for_bytes(source.as_bytes()));
}

#[test]
fn search_never_returns_an_empty_match_span() {
    let document = Document::parse("alpha beta").unwrap();

    assert!(search::search(&document, &SearchQuery::literal("")).is_empty());
    assert!(search::search(&document, &SearchQuery::literal("missing")).is_empty());
    for found in search::search(&document, &SearchQuery::literal("a")) {
        assert!(found.match_span.byte_start < found.match_span.byte_end);
    }
}

#[test]
fn ignore_case_etag_hashes_the_original_unicode_scalar() {
    let document = Document::parse("İX").unwrap();
    let matches = search::search(&document, &SearchQuery::literal_ignore_case("i"));

    assert_eq!(matches.len(), 1);
    let found = &matches[0];
    let source = exact_source(
        &document,
        found.match_span.byte_start,
        found.match_span.byte_end,
    );
    assert_eq!(source, "İ");
    assert_eq!(found.etag, TargetEtag::for_bytes("İ".as_bytes()));
}

#[test]
fn overlapping_matches_hash_each_exact_slice() {
    let document = Document::parse("aaaa").unwrap();
    let matches = search::search(&document, &SearchQuery::literal("aa"));

    assert_eq!(matches.len(), 3);
    for found in matches {
        let source = exact_source(
            &document,
            found.match_span.byte_start,
            found.match_span.byte_end,
        );
        assert_eq!(source, "aa");
        assert_eq!(found.etag, TargetEtag::for_bytes(source.as_bytes()));
    }
}

#[test]
fn identical_search_text_shares_etag_but_not_span() {
    let document = Document::parse("same same").unwrap();
    let matches = search::search(&document, &SearchQuery::literal("same"));

    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].etag, matches[1].etag);
    assert_ne!(matches[0].match_span, matches[1].match_span);
}

#[test]
fn target_etag_serializes_as_its_wire_string() {
    let etag = TargetEtag::for_bytes(b"target");
    assert_eq!(
        serde_json::to_value(&etag).unwrap(),
        serde_json::Value::String(etag.to_string())
    );
}

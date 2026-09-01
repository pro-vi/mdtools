use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;
use mdtools::patch::ReplaceBlockTarget;
use mdtools::target::{QueryResult, TargetQuery};
use mdtools::{BlockKind, SearchMatchMode};

#[test]
fn source_gap_search_returns_exact_targetless_evidence() {
    let source = "# Root\n\nvisible\n\n[^lost]: hidden needle\n\nafter\n";
    let document = Document::parse(source).unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: vec![BlockKind::Heading],
            include_source_gaps: true,
        })
        .unwrap();
    let [QueryResult::SourceEvidence { evidence }] = results.as_slice() else {
        panic!("only targetless source evidence should match: {results:?}")
    };
    assert_eq!(document.slice(&evidence.span).unwrap(), "needle");
    assert_eq!(&evidence.revision, document.revision());
    assert_eq!(evidence.etag, TargetEtag::for_bytes(b"needle"));

    let wire = serde_json::to_value(&results[0]).unwrap();
    assert!(wire["evidence"].get("target").is_none());
    assert!(wire["evidence"].get("address").is_none());
    assert!(wire["evidence"].get("guard").is_none());
    assert_eq!(
        serde_json::from_value::<QueryResult>(wire.clone()).unwrap(),
        results[0]
    );
    assert!(serde_json::from_value::<ReplaceBlockTarget>(wire["evidence"].clone()).is_err());
    let mut forbidden_target = wire;
    forbidden_target["evidence"]["target"] = serde_json::json!({"kind": "document"});
    assert!(serde_json::from_value::<QueryResult>(forbidden_target).is_err());
}

#[test]
fn source_gap_inclusion_is_explicit_and_independent_of_block_filters() {
    let source = "visible needle\n\n[^lost]: hidden needle\n";
    let document = Document::parse(source).unwrap();
    let without_gaps = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: vec![BlockKind::Paragraph],
            include_source_gaps: false,
        })
        .unwrap();
    assert_eq!(without_gaps.len(), 1);
    assert!(matches!(without_gaps[0], QueryResult::Evidence { .. }));

    let with_gaps = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: vec![BlockKind::Heading],
            include_source_gaps: true,
        })
        .unwrap();
    assert_eq!(with_gaps.len(), 1);
    assert!(matches!(with_gaps[0], QueryResult::SourceEvidence { .. }));
}

#[test]
fn search_never_crosses_source_region_boundaries() {
    let document = Document::parse("body\n\n[^lost]: note\n").unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "body\n\n[^lost]".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
        })
        .unwrap();
    assert!(results.is_empty());

    let boundary = Document::parse("# Root\n\nbody").unwrap();
    let results = boundary
        .query(&TargetQuery::Search {
            text: "\n\n".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
        })
        .unwrap();
    assert!(results.is_empty());
}

#[test]
fn referenced_footnotes_remain_target_backed_evidence() {
    let document = Document::parse("body[^kept]\n\n[^kept]: retained needle\n").unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
        })
        .unwrap();
    let [QueryResult::Evidence { evidence }] = results.as_slice() else {
        panic!("referenced footnote should remain structural: {results:?}")
    };
    assert_eq!(&evidence.revision, document.revision());
    assert_eq!(document.slice(&evidence.span).unwrap(), "needle");
}

#[test]
fn mixed_evidence_families_remain_in_strict_source_order() {
    let document =
        Document::parse("first needle\n\n[^lost]: hidden needle\n\nlast needle\n").unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
        })
        .unwrap();
    let starts = results
        .iter()
        .map(|result| match result {
            QueryResult::Evidence { evidence } => evidence.span.byte_start,
            QueryResult::SourceEvidence { evidence } => evidence.span.byte_start,
            QueryResult::Target { .. } => panic!("search returned a mutable target"),
        })
        .collect::<Vec<_>>();
    assert!(
        starts.windows(2).all(|pair| pair[0] < pair[1]),
        "{starts:?}"
    );
    assert!(matches!(results[0], QueryResult::Evidence { .. }));
    assert!(matches!(results[1], QueryResult::SourceEvidence { .. }));
    assert!(matches!(results[2], QueryResult::Evidence { .. }));
}

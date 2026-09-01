use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;
use mdtools::patch::{
    BlockInsertionTarget, FrontmatterPatchTarget, HeadingPatchTarget, Patch, PreamblePatchTarget,
    ReplaceBlockTarget, SectionInsertionTarget, SectionPatchTarget, TablePatchTarget,
    TableRowPatchTarget, TaskPatchTarget,
};
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
            max_results: 100,
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
    assert_rejects_all_patch_authorities(&wire["evidence"]);
    let mut forbidden_target = wire;
    forbidden_target["evidence"]["target"] = serde_json::json!({"kind": "document"});
    assert!(serde_json::from_value::<QueryResult>(forbidden_target).is_err());
}

fn assert_rejects_all_patch_authorities(value: &serde_json::Value) {
    fn rejects<T: serde::de::DeserializeOwned>(value: &serde_json::Value) {
        assert!(serde_json::from_value::<T>(value.clone()).is_err());
    }

    rejects::<ReplaceBlockTarget>(value);
    rejects::<BlockInsertionTarget>(value);
    rejects::<SectionPatchTarget>(value);
    rejects::<HeadingPatchTarget>(value);
    rejects::<SectionInsertionTarget>(value);
    rejects::<PreamblePatchTarget>(value);
    rejects::<TaskPatchTarget>(value);
    rejects::<TableRowPatchTarget>(value);
    rejects::<TablePatchTarget>(value);
    rejects::<FrontmatterPatchTarget>(value);
    rejects::<Patch>(value);
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
            max_results: 100,
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
            max_results: 100,
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
            max_results: 100,
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
            max_results: 100,
        })
        .unwrap();
    assert!(results.is_empty());
}

#[test]
fn newline_ending_match_uses_the_document_span_convention() {
    let document = Document::parse("body\n\n[^lost]: note\n").unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "\n\n".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
            max_results: 100,
        })
        .unwrap();
    let [QueryResult::SourceEvidence { evidence }] = results.as_slice() else {
        panic!("gap whitespace should produce one source-evidence match: {results:?}")
    };
    let expected = document.span_for_byte_range(evidence.span.byte_start, evidence.span.byte_end);
    assert_eq!(evidence.span, expected);
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
            max_results: 100,
        })
        .unwrap();
    let [QueryResult::Evidence { evidence }] = results.as_slice() else {
        panic!("referenced footnote should remain structural: {results:?}")
    };
    assert_eq!(&evidence.revision, document.revision());
    assert_eq!(document.slice(&evidence.span).unwrap(), "needle");
}

#[test]
fn link_reference_definitions_return_targetless_source_evidence() {
    let document =
        Document::parse("body [known]\n\n[known]: https://example.com \"needle\"\n").unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
            max_results: 100,
        })
        .unwrap();
    let [QueryResult::SourceEvidence { evidence }] = results.as_slice() else {
        panic!("link definition should be targetless source evidence: {results:?}")
    };
    assert_eq!(document.slice(&evidence.span).unwrap(), "needle");
    assert_eq!(evidence.preview, "[known]: https://example.com \"needle\"");
    assert_eq!(&evidence.revision, document.revision());
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
            max_results: 100,
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

#[test]
fn search_result_budget_fails_without_partial_output() {
    let document =
        Document::parse("first needle\n\n[^lost]: hidden needle needle\n\nlast needle\n").unwrap();
    let error = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
            max_results: 2,
        })
        .unwrap_err();
    assert!(matches!(
        error,
        mdtools::core_error::CoreError::SearchResultLimitExceeded { limit: 2 }
    ));

    let exact = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
            max_results: 4,
        })
        .unwrap();
    assert_eq!(exact.len(), 4);
}

#[test]
fn repetitive_source_gap_stops_at_the_explicit_result_budget() {
    let source = format!("[^lost]: {}\n", "a".repeat(100_000));
    let document = Document::parse(source).unwrap();
    assert!(matches!(
        document.query(&TargetQuery::Search {
            text: "aa".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: true,
            max_results: 2,
        }),
        Err(mdtools::core_error::CoreError::SearchResultLimitExceeded { limit: 2 })
    ));
}

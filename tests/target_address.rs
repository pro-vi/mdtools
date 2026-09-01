use std::collections::HashSet;

use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::target::{
    GuardAuthority, HeadingAddressSegment, LinkParentAddress, SectionAddress, TargetAddress,
    TargetKind, TargetQuery, TargetSummary,
};
use mdtools::{BlockKind, HeadingMatchMode, SearchMatchMode, TaskStatus};

const SOURCE: &str = "---\ntitle: Demo\nnested:\n  state: open\n---\n\nlead [pre](pre.md)\n\n# Root\n\nintro [guide](guide.md)\n\n- [ ] task [task](task.md)\n\n| Name | State |\n| --- | --- |\n| A | open |\n\n## Same\n\none\n\n## Same\n\ntwo\n\n# Other\n\n## Same\n\nthree\n";

fn heading(text: &str, occurrence: u32) -> HeadingAddressSegment {
    HeadingAddressSegment {
        text: text.into(),
        occurrence,
    }
}

#[test]
fn every_mapped_address_is_unique_and_round_trips_through_resolution() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let map = document.map().unwrap();
    let mut addresses = HashSet::new();

    for snapshot in &map {
        assert!(
            addresses.insert(snapshot.address.clone()),
            "duplicate address: {}",
            snapshot.address
        );
        assert_eq!(
            document.resolve(&snapshot.address).unwrap().snapshot(),
            snapshot
        );
    }

    assert!(map.iter().any(|snapshot| {
        snapshot.address
            == TargetAddress::Section {
                path: vec![heading("Root", 1), heading("Same", 1)],
            }
    }));
    assert!(map.iter().any(|snapshot| {
        snapshot.address
            == TargetAddress::Section {
                path: vec![heading("Root", 1), heading("Same", 2)],
            }
    }));
    assert!(map.iter().any(|snapshot| {
        snapshot.address
            == TargetAddress::Section {
                path: vec![heading("Other", 1), heading("Same", 1)],
            }
    }));
}

#[test]
fn fuzzy_multi_match_query_cannot_silently_become_one_address() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let query = TargetQuery::Section {
        text: "Same".into(),
        match_mode: HeadingMatchMode::Exact,
    };
    let matches = document.query(&query).unwrap();
    assert_eq!(matches.len(), 3);
    assert!(matches.iter().all(|result| result
        .target()
        .is_some_and(|target| target.kind == TargetKind::Section)));
    assert!(matches!(
        document.query_one(&query),
        Err(CoreError::AmbiguousTargetQuery { count: 3 })
    ));
}

#[test]
fn empty_contains_query_is_rejected_but_exact_empty_heading_is_valid() {
    let document = Document::parse("#\n").unwrap();
    for match_mode in [
        HeadingMatchMode::Contains,
        HeadingMatchMode::ContainsIgnoreCase,
    ] {
        let query = TargetQuery::Section {
            text: String::new(),
            match_mode,
        };
        assert!(matches!(
            document.query(&query),
            Err(CoreError::InvalidSelector(_))
        ));
        assert!(matches!(
            document.query_one(&query),
            Err(CoreError::InvalidSelector(_))
        ));
    }

    let exact = document
        .query_one(&TargetQuery::Section {
            text: String::new(),
            match_mode: HeadingMatchMode::Exact,
        })
        .unwrap();
    assert_eq!(exact.snapshot().kind, TargetKind::Section);
}

#[test]
fn locate_returns_identical_snapshots_for_overlapping_targets() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let map = document.map().unwrap();
    let offset = SOURCE.find("task.md").unwrap() as u32;
    let located = document.locate_targets(offset).unwrap();
    let kinds = located
        .iter()
        .map(|snapshot| snapshot.kind)
        .collect::<HashSet<_>>();

    for expected in [
        TargetKind::Document,
        TargetKind::Section,
        TargetKind::Block,
        TargetKind::Task,
        TargetKind::Link,
    ] {
        assert!(kinds.contains(&expected), "missing {expected:?}");
    }
    for snapshot in located {
        assert_eq!(
            map.iter()
                .find(|mapped| mapped.address == snapshot.address)
                .unwrap(),
            &snapshot
        );
    }
}

#[test]
fn guard_authority_is_explicit_and_can_differ_from_the_selected_span() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let map = document.map().unwrap();
    let row = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::TableRow)
        .unwrap();
    let GuardAuthority::Container { span, address, .. } = &row.guard else {
        panic!("table row must guard its table")
    };
    assert_ne!(*span, row.selection_span.unwrap());
    assert!(matches!(address.as_ref(), TargetAddress::Block { .. }));

    let field = map
        .iter()
        .find(|snapshot| {
            snapshot.address
                == TargetAddress::FrontmatterField {
                    path: vec!["nested".into(), "state".into()],
                }
        })
        .unwrap();
    assert!(matches!(field.guard, GuardAuthority::Frontmatter { .. }));
}

#[test]
fn frontmatter_path_segments_distinguish_literal_dots_from_nesting() {
    let document =
        Document::parse_for_frontmatter("---\n\"a.b\": literal\na:\n  b: nested\n---\nbody\n")
            .unwrap();
    let literal = TargetAddress::FrontmatterField {
        path: vec!["a.b".into()],
    };
    let nested = TargetAddress::FrontmatterField {
        path: vec!["a".into(), "b".into()],
    };
    assert_ne!(literal, nested);
    assert_eq!(
        document
            .resolve(&literal)
            .unwrap()
            .read_frontmatter_field(&document)
            .unwrap()
            .value,
        Some(serde_json::json!("literal"))
    );
    assert_eq!(
        document
            .resolve(&nested)
            .unwrap()
            .read_frontmatter_field(&document)
            .unwrap()
            .value,
        Some(serde_json::json!("nested"))
    );
}

#[test]
fn empty_frontmatter_keys_are_exactly_addressable() {
    let document = Document::parse_for_frontmatter("---\n\"\": value\n---\n").unwrap();
    let mapped = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            snapshot.address
                == TargetAddress::FrontmatterField {
                    path: vec![String::new()],
                }
        })
        .unwrap();
    let resolved = document.resolve(&mapped.address).unwrap();
    assert_eq!(resolved.snapshot(), &mapped);
    assert_eq!(
        resolved.read_frontmatter_field(&document).unwrap().value,
        Some(serde_json::json!("value"))
    );
}

#[test]
fn address_deserialization_rejects_unknown_fields_at_every_layer() {
    for json in [
        r#"{"kind":"document","etag":"ignored"}"#,
        r#"{"kind":"section","path":[{"text":"A","occurrence":1,"revision":"ignored"}]}"#,
        r#"{"kind":"block","block":{"section":{"kind":"preamble"},"ordinal":0,"extra":true}}"#,
        r#"{"kind":"block","block":{"section":{"kind":"preamble","extra":true},"ordinal":0}}"#,
        r#"{"kind":"link","parent":{"kind":"heading","section":{"kind":"heading","path":[{"text":"A","occurrence":1}]},"extra":true},"occurrence":0}"#,
    ] {
        assert!(
            serde_json::from_str::<TargetAddress>(json).is_err(),
            "{json}"
        );
    }
}

#[test]
fn impossible_address_shapes_fail_before_lookup() {
    let document = Document::parse("# A\n").unwrap();
    let invalid = [TargetAddress::Task {
        block: mdtools::target::BlockAddress {
            section: SectionAddress::Preamble,
            ordinal: 0,
        },
        path: Vec::new(),
    }];
    for address in invalid {
        assert!(
            serde_json::from_value::<TargetAddress>(serde_json::to_value(&address).unwrap())
                .is_err()
        );
        assert!(matches!(
            document.resolve(&address),
            Err(CoreError::InvalidTargetAddress { .. })
        ));
    }
    assert!(serde_json::from_value::<TargetAddress>(serde_json::json!({
        "kind": "link",
        "parent": { "kind": "heading", "section": { "kind": "preamble" } },
        "occurrence": 0
    }))
    .is_err());
}

#[test]
fn addresses_serialize_identity_without_observed_state() {
    let address = TargetAddress::Task {
        block: mdtools::target::BlockAddress {
            section: SectionAddress::Heading {
                path: vec![heading("Root", 1)],
            },
            ordinal: 1,
        },
        path: vec![0],
    };
    let json = serde_json::to_value(&address).unwrap();
    let rendered = json.to_string();
    assert!(!rendered.contains("etag"));
    assert!(!rendered.contains("revision"));
    assert_eq!(
        serde_json::from_value::<TargetAddress>(json).unwrap(),
        address
    );
}

#[test]
fn task_query_filters_snapshots_without_creating_addresses() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let matches = document
        .query(&TargetQuery::Task {
            status: Some(TaskStatus::Pending),
            contains: Some("task".into()),
        })
        .unwrap();
    assert_eq!(matches.len(), 1);
    assert!(matches!(
        matches[0].target().unwrap().summary,
        TargetSummary::Task {
            status: TaskStatus::Pending,
            ..
        }
    ));
}

#[test]
fn representative_fixtures_have_no_address_collisions() {
    for (name, source) in [
        ("nested_tasks", include_str!("fixtures/nested_tasks.md")),
        ("footnote", include_str!("fixtures/footnote_midbody.md")),
        ("setext", include_str!("fixtures/setext.md")),
        ("table", include_str!("fixtures/table.md")),
        ("frontmatter", include_str!("fixtures/frontmatter.md")),
    ] {
        let document = Document::parse_for_frontmatter(source).unwrap();
        let map = document.map().unwrap();
        let addresses = map
            .iter()
            .map(|snapshot| snapshot.address.clone())
            .collect::<HashSet<_>>();
        assert_eq!(addresses.len(), map.len(), "{name}");
        for snapshot in map {
            assert_eq!(
                document.resolve(&snapshot.address).unwrap().snapshot(),
                &snapshot,
                "{name}: {}",
                snapshot.address
            );
        }
    }
}

#[test]
fn unique_query_and_map_return_identical_evidence() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let query = TargetQuery::Section {
        text: "Root".into(),
        match_mode: HeadingMatchMode::Exact,
    };
    let queried = document.query_one(&query).unwrap();
    let mapped = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.address == *queried.address())
        .unwrap();
    assert_eq!(queried.snapshot(), &mapped);
}

#[test]
fn heading_links_use_the_complete_heading_as_their_parent_address() {
    for source in [
        "# [ATX](atx.md)\n",
        "[Setext](setext.md)\n==================\n",
    ] {
        let document = Document::parse(source).unwrap();
        let link = document
            .map()
            .unwrap()
            .into_iter()
            .find(|snapshot| snapshot.kind == TargetKind::Link)
            .unwrap();
        assert!(matches!(
            link.address,
            TargetAddress::Link {
                parent: LinkParentAddress::Heading { .. },
                occurrence: 0,
            }
        ));
    }
}

#[test]
fn table_row_line_ending_positions_keep_the_same_snapshot() {
    let source = "| h |\r\n|---|\r\n| v |\r\n";
    let document = Document::parse(source).unwrap();
    let row = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::TableRow)
        .unwrap();
    let row_span = row.selection_span.unwrap();
    for offset in [row_span.byte_end, row_span.byte_end + 1] {
        assert_eq!(
            document
                .locate_targets(offset)
                .unwrap()
                .into_iter()
                .find(|snapshot| snapshot.kind == TargetKind::TableRow)
                .unwrap(),
            row
        );
    }
}

#[test]
fn search_query_returns_evidence_that_cannot_resolve_as_mutation_authority() {
    let document = Document::parse("# Work\n\nfind needle here\n").unwrap();
    let query = TargetQuery::Search {
        text: "needle".into(),
        match_mode: SearchMatchMode::Literal,
        block_kinds: vec![BlockKind::Paragraph],
        include_source_gaps: false,
        max_results: 100,
    };
    let results = document.query(&query).unwrap();
    let [result] = results.as_slice() else {
        panic!("one search result")
    };
    let evidence = result.evidence().expect("search yields evidence");
    assert_eq!(document.slice(&evidence.span).unwrap(), "needle");
    assert!(document.resolve(&evidence.target).is_ok());
    assert!(matches!(
        document.query_one(&query),
        Err(CoreError::InvalidSelector(reason))
            if reason.contains("cannot resolve as one mutable target")
    ));
}

#[test]
fn search_evidence_is_returned_in_source_order_with_footnotes() {
    let document =
        Document::parse("needle top\n\n[^1]: needle foot\n\n# Later\n\nneedle later[^1]\n")
            .unwrap();
    let results = document
        .query(&TargetQuery::Search {
            text: "needle".into(),
            match_mode: SearchMatchMode::Literal,
            block_kinds: Vec::new(),
            include_source_gaps: false,
            max_results: 100,
        })
        .unwrap();
    let starts = results
        .iter()
        .map(|result| result.evidence().unwrap().span.byte_start)
        .collect::<Vec<_>>();
    assert!(
        starts.windows(2).all(|pair| pair[0] < pair[1]),
        "{starts:?}"
    );
}

#[test]
fn excessive_blockquote_nesting_returns_a_typed_parse_error() {
    let source = format!("{} deep\n", ">".repeat(2048));
    assert!(matches!(
        Document::parse(source),
        Err(CoreError::ParseFailed(reason)) if reason.contains("AST exceeds")
    ));
}
